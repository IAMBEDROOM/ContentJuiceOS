use std::time::Instant;

use serde::Deserialize;
use sha2::Digest;
use tokio::sync::{oneshot, Mutex};

use crate::platform::error::{PlatformError, PlatformResult};
use crate::platform::twitch::oauth::OAuthCallbackParams;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Bundled Kick credentials — injected at compile time from `src-tauri/.env`.
pub const KICK_CLIENT_ID: &str = env!("KICK_CLIENT_ID");
pub const KICK_CLIENT_SECRET: &str = env!("KICK_CLIENT_SECRET");

pub const KICK_AUTH_URL: &str = "https://id.kick.com/oauth/authorize";
pub const KICK_TOKEN_URL: &str = "https://id.kick.com/oauth/token";
pub const KICK_REVOKE_URL: &str = "https://id.kick.com/oauth/revoke";
pub const KICK_INTROSPECT_URL: &str = "https://api.kick.com/token/introspect";

/// Phase 1-2 scopes: user info, channel info, event subscriptions.
/// Phase 3 will add chat scopes — users re-auth and the upsert handles it.
pub const KICK_SCOPES: &[&str] = &[
    "user:read",
    "channel:read",
    "events:subscribe",
];

/// Valid redirect port range. These ports must be registered on the Kick Developer Console.
pub const KICK_REDIRECT_PORTS: std::ops::RangeInclusive<u16> = 4848..=4868;

// ---------------------------------------------------------------------------
// Auth state (CSRF + callback signaling)
// ---------------------------------------------------------------------------

pub struct KickAuthState {
    pending: Mutex<Option<PendingOAuthFlow>>,
}

struct PendingOAuthFlow {
    state_param: String,
    result_tx: oneshot::Sender<Result<OAuthCallbackParams, String>>,
    created_at: Instant,
}

impl KickAuthState {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(None),
        }
    }

    /// Store a pending flow. Returns Err if one is already in progress.
    pub async fn set_pending(
        &self,
        state_param: String,
        result_tx: oneshot::Sender<Result<OAuthCallbackParams, String>>,
    ) -> PlatformResult<()> {
        let mut pending = self.pending.lock().await;

        // Clean up expired flows (> 5 minutes old)
        if let Some(existing) = pending.as_ref() {
            if existing.created_at.elapsed().as_secs() > 300 {
                *pending = None;
            } else {
                return Err(PlatformError::InvalidState(
                    "An OAuth flow is already in progress. Please complete or cancel it first."
                        .to_string(),
                ));
            }
        }

        *pending = Some(PendingOAuthFlow {
            state_param,
            result_tx,
            created_at: Instant::now(),
        });
        Ok(())
    }

    /// Called by the HTTP callback handler to deliver the result.
    pub async fn complete_pending(
        &self,
        params: Result<OAuthCallbackParams, String>,
    ) -> PlatformResult<()> {
        let mut pending = self.pending.lock().await;
        match pending.take() {
            Some(flow) => {
                // Verify state parameter matches (CSRF protection)
                if let Ok(ref p) = params {
                    if p.state != flow.state_param {
                        let _ = flow
                            .result_tx
                            .send(Err("State parameter mismatch — possible CSRF attack".into()));
                        return Err(PlatformError::InvalidState(
                            "State parameter mismatch".to_string(),
                        ));
                    }
                }
                let _ = flow.result_tx.send(params);
                Ok(())
            }
            None => Err(PlatformError::InvalidState(
                "No pending OAuth flow to complete".to_string(),
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Token response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct KickTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    /// Kick returns scope as a space-separated string (not an array like Twitch).
    pub scope: String,
    pub token_type: String,
}

/// Response from Kick's token introspection endpoint.
/// Used instead of a "get current user" endpoint (which Kick doesn't have).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct KickIntrospectResponse {
    pub active: bool,
    pub sub: Option<String>,
    pub scope: Option<String>,
    pub exp: Option<u64>,
}

// ---------------------------------------------------------------------------
// OAuth functions
// ---------------------------------------------------------------------------

/// Constructs the Kick authorization URL for the browser.
/// `code_challenge` is the S256-hashed PKCE code verifier.
pub fn build_authorize_url(redirect_uri: &str, state: &str, code_challenge: &str) -> String {
    let scopes = KICK_SCOPES.join(" ");
    format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}\
         &code_challenge={}&code_challenge_method=S256",
        KICK_AUTH_URL,
        KICK_CLIENT_ID,
        urlencoding_encode(redirect_uri),
        urlencoding_encode(&scopes),
        state,
        code_challenge,
    )
}

/// Generate a PKCE code verifier (43-128 chars of unreserved characters).
pub fn generate_code_verifier() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    base64_url_encode(&bytes)
}

/// Compute the S256 code challenge from a code verifier.
pub fn compute_code_challenge(verifier: &str) -> String {
    let hash = sha2::Sha256::digest(verifier.as_bytes());
    base64_url_encode(&hash)
}

/// Base64url encoding (no padding) per RFC 7636.
fn base64_url_encode(input: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(input)
}

/// Exchange an authorization code for access + refresh tokens using PKCE.
pub async fn exchange_code_for_tokens(
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> PlatformResult<KickTokenResponse> {
    eprintln!(
        "[KICK DEBUG] Token exchange — client_id: '{}', redirect_uri: '{}', code_verifier length: {}",
        KICK_CLIENT_ID,
        redirect_uri,
        code_verifier.len(),
    );

    let client = reqwest::Client::new();
    let response = client
        .post(KICK_TOKEN_URL)
        .form(&[
            ("client_id", KICK_CLIENT_ID),
            ("client_secret", KICK_CLIENT_SECRET),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri),
            ("code_verifier", code_verifier),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(PlatformError::OAuth(format!(
            "Token exchange failed (HTTP {status}): {body}"
        )));
    }

    response
        .json::<KickTokenResponse>()
        .await
        .map_err(|e| PlatformError::OAuth(format!("Failed to parse token response: {e}")))
}

/// Refresh an access token using a refresh token.
pub async fn refresh_access_token(
    refresh_token: &str,
) -> PlatformResult<KickTokenResponse> {
    let client = reqwest::Client::new();
    let response = client
        .post(KICK_TOKEN_URL)
        .form(&[
            ("client_id", KICK_CLIENT_ID),
            ("client_secret", KICK_CLIENT_SECRET),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(PlatformError::OAuth(format!(
            "Token refresh failed (HTTP {status}): {body}"
        )));
    }

    response
        .json::<KickTokenResponse>()
        .await
        .map_err(|e| PlatformError::OAuth(format!("Failed to parse refresh response: {e}")))
}

/// Introspect a token to get user information.
/// Kick has no "get current user by token" endpoint, so we use introspection
/// to retrieve the `sub` field (user ID).
pub async fn introspect_token(access_token: &str) -> PlatformResult<KickIntrospectResponse> {
    let client = reqwest::Client::new();
    let url = format!("{}?access_token={}", KICK_INTROSPECT_URL, access_token);
    let response = client
        .post(&url)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(PlatformError::OAuth(format!(
            "Token introspection failed (HTTP {status}): {body}"
        )));
    }

    let introspect: KickIntrospectResponse = response
        .json()
        .await
        .map_err(|e| PlatformError::OAuth(format!("Failed to parse introspect response: {e}")))?;

    if !introspect.active {
        return Err(PlatformError::OAuth(
            "Token is not active according to introspection".to_string(),
        ));
    }

    Ok(introspect)
}

/// Revoke an access token.
/// Kick uses query parameters for revocation (not form body like Twitch).
pub async fn revoke_token(access_token: &str) -> PlatformResult<()> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}?token={}&token_hint_type=access_token",
        KICK_REVOKE_URL, access_token
    );
    let response = client
        .post(&url)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(PlatformError::OAuth(format!(
            "Token revocation failed (HTTP {status}): {body}"
        )));
    }

    Ok(())
}

/// Check if a token is expired (or will expire within `buffer_seconds`).
#[allow(dead_code)]
pub fn is_token_expired(expires_at: &str, buffer_seconds: i64) -> bool {
    match chrono::DateTime::parse_from_rfc3339(expires_at) {
        Ok(expiry) => {
            let now = chrono::Utc::now();
            let buffered = expiry - chrono::Duration::seconds(buffer_seconds);
            now >= buffered
        }
        Err(_) => true, // If we can't parse the expiry, treat as expired
    }
}

/// Simple percent-encoding for URL query parameters.
fn urlencoding_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_authorize_url_contains_required_params() {
        let url = build_authorize_url(
            "http://localhost:4848/auth/callback/kick",
            "test_state",
            "test_challenge",
        );
        assert!(url.starts_with(KICK_AUTH_URL));
        assert!(url.contains(&format!("client_id={}", KICK_CLIENT_ID)));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("state=test_state"));
        assert!(url.contains("user%3Aread"));
        assert!(url.contains("channel%3Aread"));
        assert!(url.contains("events%3Asubscribe"));
        assert!(url.contains("code_challenge=test_challenge"));
        assert!(url.contains("code_challenge_method=S256"));
    }

    #[test]
    fn pkce_verifier_and_challenge_are_valid() {
        let verifier = generate_code_verifier();
        assert!(verifier.len() >= 43); // RFC 7636 minimum
        let challenge = compute_code_challenge(&verifier);
        assert!(!challenge.is_empty());
        assert!(!challenge.contains('=')); // No padding in base64url
        assert!(!challenge.contains('+'));
        assert!(!challenge.contains('/'));
    }

    #[test]
    fn is_token_expired_returns_true_for_past() {
        assert!(is_token_expired("2020-01-01T00:00:00Z", 0));
    }

    #[test]
    fn is_token_expired_returns_false_for_future() {
        assert!(!is_token_expired("2099-01-01T00:00:00Z", 0));
    }

    #[test]
    fn is_token_expired_respects_buffer() {
        assert!(!is_token_expired("2099-01-01T00:00:00Z", 300));
    }

    #[test]
    fn is_token_expired_invalid_date_treated_as_expired() {
        assert!(is_token_expired("not-a-date", 0));
    }

    #[test]
    fn urlencoding_encodes_special_chars() {
        assert_eq!(urlencoding_encode("user:read"), "user%3Aread");
        assert_eq!(urlencoding_encode("hello world"), "hello%20world");
    }
}

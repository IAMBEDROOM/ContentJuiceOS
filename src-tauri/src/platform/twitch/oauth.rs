use std::time::Instant;

use serde::Deserialize;
use sha2::Digest;
use tokio::sync::{oneshot, Mutex};

use crate::platform::error::{PlatformError, PlatformResult};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Bundled Twitch credentials — injected at compile time from `src-tauri/.env`.
/// Twitch requires both Client ID and Client Secret for the authorization code flow,
/// even with PKCE. PKCE still adds security against authorization code interception.
pub const TWITCH_CLIENT_ID: &str = env!("TWITCH_CLIENT_ID");
pub const TWITCH_CLIENT_SECRET: &str = env!("TWITCH_CLIENT_SECRET");

pub const TWITCH_AUTH_URL: &str = "https://id.twitch.tv/oauth2/authorize";
pub const TWITCH_TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";
#[allow(dead_code)]
pub const TWITCH_VALIDATE_URL: &str = "https://id.twitch.tv/oauth2/validate";
pub const TWITCH_USERS_URL: &str = "https://api.twitch.tv/helix/users";
pub const TWITCH_REVOKE_URL: &str = "https://id.twitch.tv/oauth2/revoke";

/// Phase 1-2 scopes: user info, subscriptions, bits.
/// Follows, raids, and gift subs are available via EventSub without additional scopes.
/// Phase 3 will add chat/moderation scopes — users re-auth and the upsert handles it.
pub const TWITCH_SCOPES: &[&str] = &["user:read:email", "channel:read:subscriptions", "bits:read"];

/// Valid redirect port range. These ports must be registered on the Twitch Developer Console.
pub const TWITCH_REDIRECT_PORTS: std::ops::RangeInclusive<u16> = 4848..=4858;

// ---------------------------------------------------------------------------
// Auth state (CSRF + callback signaling)
// ---------------------------------------------------------------------------

pub struct TwitchAuthState {
    pending: Mutex<Option<PendingOAuthFlow>>,
}

struct PendingOAuthFlow {
    state_param: String,
    result_tx: oneshot::Sender<Result<OAuthCallbackParams, String>>,
    created_at: Instant,
}

impl TwitchAuthState {
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
// OAuth callback params
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct OAuthCallbackParams {
    pub code: Option<String>,
    pub state: String,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

// ---------------------------------------------------------------------------
// Token response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TwitchTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub scope: Vec<String>,
    pub token_type: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TwitchUser {
    pub id: String,
    pub login: String,
    pub display_name: String,
    pub profile_image_url: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TwitchUsersResponse {
    data: Vec<TwitchUser>,
}

// ---------------------------------------------------------------------------
// OAuth functions
// ---------------------------------------------------------------------------

/// Constructs the Twitch authorization URL for the browser.
/// `code_challenge` is the S256-hashed PKCE code verifier.
pub fn build_authorize_url(redirect_uri: &str, state: &str, code_challenge: &str) -> String {
    let scopes = TWITCH_SCOPES.join(" ");
    format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&force_verify=true\
         &code_challenge={}&code_challenge_method=S256",
        TWITCH_AUTH_URL,
        TWITCH_CLIENT_ID,
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
) -> PlatformResult<TwitchTokenResponse> {
    eprintln!(
        "[TWITCH DEBUG] Token exchange — client_id: '{}', redirect_uri: '{}', code_verifier length: {}",
        TWITCH_CLIENT_ID,
        redirect_uri,
        code_verifier.len(),
    );

    let client = reqwest::Client::new();
    let response = client
        .post(TWITCH_TOKEN_URL)
        .form(&[
            ("client_id", TWITCH_CLIENT_ID),
            ("client_secret", TWITCH_CLIENT_SECRET),
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
        .json::<TwitchTokenResponse>()
        .await
        .map_err(|e| PlatformError::OAuth(format!("Failed to parse token response: {e}")))
}

/// Refresh an access token using a refresh token.
pub async fn refresh_access_token(refresh_token: &str) -> PlatformResult<TwitchTokenResponse> {
    let client = reqwest::Client::new();
    let response = client
        .post(TWITCH_TOKEN_URL)
        .form(&[
            ("client_id", TWITCH_CLIENT_ID),
            ("client_secret", TWITCH_CLIENT_SECRET),
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
        .json::<TwitchTokenResponse>()
        .await
        .map_err(|e| PlatformError::OAuth(format!("Failed to parse refresh response: {e}")))
}

/// Fetch the current user's profile using a valid access token.
pub async fn get_current_user(access_token: &str) -> PlatformResult<TwitchUser> {
    let client = reqwest::Client::new();
    let response = client
        .get(TWITCH_USERS_URL)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("Client-Id", TWITCH_CLIENT_ID)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(PlatformError::OAuth(format!(
            "Failed to fetch user (HTTP {status}): {body}"
        )));
    }

    let users: TwitchUsersResponse = response
        .json()
        .await
        .map_err(|e| PlatformError::OAuth(format!("Failed to parse user response: {e}")))?;

    users
        .data
        .into_iter()
        .next()
        .ok_or_else(|| PlatformError::OAuth("No user returned from Twitch API".to_string()))
}

/// Revoke an access token.
pub async fn revoke_token(access_token: &str) -> PlatformResult<()> {
    let client = reqwest::Client::new();
    let response = client
        .post(TWITCH_REVOKE_URL)
        .form(&[("client_id", TWITCH_CLIENT_ID), ("token", access_token)])
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
            "http://localhost:4848/auth/callback/twitch",
            "test_state",
            "test_challenge",
        );
        assert!(url.starts_with(TWITCH_AUTH_URL));
        assert!(url.contains(&format!("client_id={}", TWITCH_CLIENT_ID)));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("state=test_state"));
        assert!(url.contains("user%3Aread%3Aemail"));
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
        // Far future, huge buffer — still not expired
        assert!(!is_token_expired("2099-01-01T00:00:00Z", 300));
    }

    #[test]
    fn is_token_expired_invalid_date_treated_as_expired() {
        assert!(is_token_expired("not-a-date", 0));
    }

    #[test]
    fn urlencoding_encodes_special_chars() {
        assert_eq!(urlencoding_encode("user:read:email"), "user%3Aread%3Aemail");
        assert_eq!(urlencoding_encode("hello world"), "hello%20world");
    }
}

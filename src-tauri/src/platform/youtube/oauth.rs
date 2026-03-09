use std::time::Instant;

use serde::Deserialize;
use sha2::Digest;
use tokio::sync::{oneshot, Mutex};

use crate::platform::error::{PlatformError, PlatformResult};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Bundled YouTube/Google credentials — injected at compile time from `src-tauri/.env`.
/// Google requires both Client ID and Client Secret for the authorization code flow.
/// PKCE is used alongside the secret for defense-in-depth.
pub const YOUTUBE_CLIENT_ID: &str = env!("YOUTUBE_CLIENT_ID");
pub const YOUTUBE_CLIENT_SECRET: &str = env!("YOUTUBE_CLIENT_SECRET");

pub const YOUTUBE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const YOUTUBE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
pub const YOUTUBE_REVOKE_URL: &str = "https://oauth2.googleapis.com/revoke";
pub const YOUTUBE_CHANNELS_URL: &str = "https://www.googleapis.com/youtube/v3/channels";

/// Phase 1-2 scopes: channel info, email, profile.
/// Phase 3+ will add live-streaming and chat scopes — users re-auth and the upsert handles it.
pub const YOUTUBE_SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/youtube.readonly",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
];

// ---------------------------------------------------------------------------
// Auth state (CSRF + callback signaling)
// ---------------------------------------------------------------------------

pub struct YouTubeAuthState {
    pending: Mutex<Option<PendingOAuthFlow>>,
}

struct PendingOAuthFlow {
    state_param: String,
    result_tx: oneshot::Sender<Result<crate::platform::twitch::oauth::OAuthCallbackParams, String>>,
    created_at: Instant,
}

impl YouTubeAuthState {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(None),
        }
    }

    /// Store a pending flow. Returns Err if one is already in progress.
    pub async fn set_pending(
        &self,
        state_param: String,
        result_tx: oneshot::Sender<
            Result<crate::platform::twitch::oauth::OAuthCallbackParams, String>,
        >,
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
        params: Result<crate::platform::twitch::oauth::OAuthCallbackParams, String>,
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

/// Google token response. `refresh_token` is `Option` because Google's refresh
/// endpoint does NOT return a new refresh token — only the initial auth does.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct YouTubeTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub scope: String, // Google returns scopes as a space-separated string
    pub token_type: String,
}

// ---------------------------------------------------------------------------
// YouTube Channels API response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct YouTubeChannelsResponse {
    pub items: Vec<YouTubeChannelItem>,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeChannelItem {
    pub id: String,
    pub snippet: YouTubeChannelSnippet,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct YouTubeChannelSnippet {
    pub title: String,
    pub custom_url: Option<String>,
    pub thumbnails: YouTubeThumbnails,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeThumbnails {
    pub default: Option<YouTubeThumbnail>,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeThumbnail {
    pub url: String,
}

// ---------------------------------------------------------------------------
// OAuth functions
// ---------------------------------------------------------------------------

/// Constructs the Google authorization URL for the browser.
/// Includes `access_type=offline` and `prompt=consent` to guarantee a refresh token.
pub fn build_authorize_url(redirect_uri: &str, state: &str, code_challenge: &str) -> String {
    let scopes = YOUTUBE_SCOPES.join(" ");
    format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}\
         &access_type=offline&prompt=consent\
         &code_challenge={}&code_challenge_method=S256",
        YOUTUBE_AUTH_URL,
        YOUTUBE_CLIENT_ID,
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
) -> PlatformResult<YouTubeTokenResponse> {
    let client = reqwest::Client::new();
    let response = client
        .post(YOUTUBE_TOKEN_URL)
        .form(&[
            ("client_id", YOUTUBE_CLIENT_ID),
            ("client_secret", YOUTUBE_CLIENT_SECRET),
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
        .json::<YouTubeTokenResponse>()
        .await
        .map_err(|e| PlatformError::OAuth(format!("Failed to parse token response: {e}")))
}

/// Refresh an access token using a refresh token.
/// **Critical:** Google does NOT return a new refresh_token on refresh — the caller
/// must preserve the original refresh_token.
pub async fn refresh_access_token(refresh_token: &str) -> PlatformResult<YouTubeTokenResponse> {
    let client = reqwest::Client::new();
    let response = client
        .post(YOUTUBE_TOKEN_URL)
        .form(&[
            ("client_id", YOUTUBE_CLIENT_ID),
            ("client_secret", YOUTUBE_CLIENT_SECRET),
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
        .json::<YouTubeTokenResponse>()
        .await
        .map_err(|e| PlatformError::OAuth(format!("Failed to parse refresh response: {e}")))
}

/// Fetch the authenticated user's YouTube channel.
/// Uses `channels?part=snippet&mine=true` which returns the channel associated
/// with the authenticated account.
pub async fn get_current_channel(access_token: &str) -> PlatformResult<YouTubeChannelItem> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}?part=snippet&mine=true", YOUTUBE_CHANNELS_URL))
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(PlatformError::OAuth(format!(
            "Failed to fetch channel (HTTP {status}): {body}"
        )));
    }

    let channels: YouTubeChannelsResponse = response
        .json()
        .await
        .map_err(|e| PlatformError::OAuth(format!("Failed to parse channel response: {e}")))?;

    channels.items.into_iter().next().ok_or_else(|| {
        PlatformError::OAuth("No YouTube channel found for this account".to_string())
    })
}

/// Revoke an access or refresh token.
/// Google's revocation sends the token as a query parameter, not form body.
pub async fn revoke_token(token: &str) -> PlatformResult<()> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}?token={}", YOUTUBE_REVOKE_URL, token))
        .header("Content-Type", "application/x-www-form-urlencoded")
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
            "http://localhost:4848/auth/callback/youtube",
            "test_state",
            "test_challenge",
        );
        assert!(url.starts_with(YOUTUBE_AUTH_URL));
        assert!(url.contains(&format!("client_id={}", YOUTUBE_CLIENT_ID)));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("state=test_state"));
        assert!(url.contains("code_challenge=test_challenge"));
        assert!(url.contains("code_challenge_method=S256"));
        // Google-specific params
        assert!(url.contains("access_type=offline"));
        assert!(url.contains("prompt=consent"));
        // Scopes should be URL-encoded
        assert!(url.contains("youtube.readonly"));
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
        assert_eq!(
            urlencoding_encode("https://www.googleapis.com/auth/youtube.readonly"),
            "https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fyoutube.readonly"
        );
        assert_eq!(urlencoding_encode("hello world"), "hello%20world");
    }
}

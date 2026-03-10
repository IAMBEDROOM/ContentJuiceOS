use std::sync::Arc;
use std::time::Duration;

use log::{error, info};
use tauri::State;
use tokio::sync::oneshot;

use crate::credentials::store::CredentialManager;
use crate::credentials::types::{CredentialKind, OAuthTokens};
use crate::db::Database;
use crate::platform::repository;
use crate::platform::types::{NewPlatformConnection, PlatformConnection};
use crate::rate_limiter::types::Priority;
use crate::rate_limiter::RateLimiterService;
use crate::retry::RetryService;
use crate::server::HttpServer;
use crate::types::Platform;
use crate::user_error::UserFacingError;

use super::oauth::{
    self, OAuthCallbackParams, TwitchAuthState, TWITCH_REDIRECT_PORTS, TWITCH_SCOPES,
};

/// Start the full Twitch OAuth2 authorization code flow.
///
/// 1. Generates a CSRF state parameter
/// 2. Opens the browser to Twitch's authorization page
/// 3. Waits for the callback (up to 5 minutes)
/// 4. Exchanges the code for tokens
/// 5. Fetches the user profile
/// 6. Stores everything securely
#[tauri::command]
pub async fn start_twitch_auth(
    auth_state: State<'_, Arc<TwitchAuthState>>,
    db: State<'_, Arc<Database>>,
    cred_manager: State<'_, CredentialManager>,
    http_server: State<'_, HttpServer>,
) -> Result<PlatformConnection, String> {
    let port = http_server.port();

    // Validate port is in the registered redirect URI range
    if !TWITCH_REDIRECT_PORTS.contains(&port) {
        return Err(format!(
            "HTTP server port {port} is outside the registered Twitch redirect URI range ({}-{}). \
             OAuth will fail because Twitch won't accept this redirect URI.",
            TWITCH_REDIRECT_PORTS.start(),
            TWITCH_REDIRECT_PORTS.end()
        ));
    }

    let redirect_uri = format!("http://localhost:{port}/auth/callback/twitch");

    // Generate PKCE code verifier + challenge
    let code_verifier = oauth::generate_code_verifier();
    let code_challenge = oauth::compute_code_challenge(&code_verifier);

    // Generate cryptographic random state parameter (CSRF protection)
    let state_param = generate_state();

    // Create oneshot channel for callback → command communication
    let (tx, rx) = oneshot::channel::<Result<OAuthCallbackParams, String>>();

    // Register the pending flow
    auth_state
        .set_pending(state_param.clone(), tx)
        .await
        .map_err(|e| e.to_string())?;

    // Build the authorization URL and open the browser
    let auth_url = oauth::build_authorize_url(&redirect_uri, &state_param, &code_challenge);
    info!("Opening Twitch authorization URL in browser");

    tauri_plugin_opener::open_url(&auth_url, None::<&str>)
        .map_err(|e| format!("Failed to open browser: {e}"))?;

    // Wait for the callback with a 5-minute timeout
    let callback_result = tokio::time::timeout(Duration::from_secs(300), rx)
        .await
        .map_err(|_| "Twitch authorization timed out after 5 minutes".to_string())?
        .map_err(|_| "Authorization flow was cancelled".to_string())?
        .map_err(|e| format!("Authorization failed: {e}"))?;

    // Check for errors from Twitch
    if let Some(error) = &callback_result.error {
        let desc = callback_result
            .error_description
            .as_deref()
            .unwrap_or("Unknown error");
        return Err(format!("Twitch denied authorization: {error} — {desc}"));
    }

    let code = callback_result
        .code
        .ok_or("No authorization code received from Twitch")?;

    // Exchange the authorization code for tokens (PKCE: send code_verifier)
    info!("Exchanging authorization code for tokens");
    let token_response = oauth::exchange_code_for_tokens(&code, &redirect_uri, &code_verifier)
        .await
        .map_user_err()?;

    // Fetch user profile
    info!("Fetching Twitch user profile");
    let user = oauth::get_current_user(&token_response.access_token)
        .await
        .map_user_err()?;

    // Calculate token expiry
    let expires_at =
        chrono::Utc::now() + chrono::Duration::seconds(token_response.expires_in as i64);

    // Upsert the platform connection
    let new_connection = NewPlatformConnection {
        platform: "twitch".to_string(),
        platform_user_id: user.id.clone(),
        platform_username: user.login.clone(),
        display_name: user.display_name.clone(),
        avatar_url: Some(user.profile_image_url.clone()),
        scopes: TWITCH_SCOPES.iter().map(|s| s.to_string()).collect(),
    };

    let connection = {
        let conn = db.conn.lock().map_user_err()?;
        repository::upsert_connection(&conn, &new_connection).map_user_err()?
    };

    // Store tokens securely
    let tokens = OAuthTokens {
        access_token: token_response.access_token,
        refresh_token: Some(token_response.refresh_token),
        token_expires_at: Some(expires_at.to_rfc3339()),
    };
    cred_manager
        .store_platform_tokens(&connection.id, &tokens)
        .map_user_err()?;

    info!(
        "Twitch connected: {} ({})",
        connection.display_name, connection.platform_user_id
    );

    Ok(connection)
}

/// Refresh Twitch tokens for an existing connection.
///
/// Uses the retry service for automatic exponential backoff on transient failures
/// and integrates with the rate limiter to respect Twitch's API limits.
#[tauri::command]
pub async fn refresh_twitch_tokens(
    connection_id: String,
    db: State<'_, Arc<Database>>,
    cred_manager: State<'_, CredentialManager>,
    retry_service: State<'_, Arc<RetryService>>,
    rate_limiter: State<'_, Arc<RateLimiterService>>,
) -> Result<(), String> {
    // Get existing tokens
    let tokens = cred_manager
        .get_platform_tokens(&connection_id)
        .map_user_err()?
        .ok_or("No tokens found for this connection")?;

    let refresh_token = tokens.refresh_token.ok_or("No refresh token available")?;

    // Refresh with retry + rate limiting
    let refresh_tok = refresh_token.clone();
    let new_tokens = retry_service
        .execute(Platform::Twitch, Priority::Realtime, &rate_limiter, || {
            let rt = refresh_tok.clone();
            async move { oauth::refresh_access_token(&rt).await }
        })
        .await
        .map_user_err()?;

    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(new_tokens.expires_in as i64);

    // Store updated tokens
    let updated = OAuthTokens {
        access_token: new_tokens.access_token,
        refresh_token: Some(new_tokens.refresh_token),
        token_expires_at: Some(expires_at.to_rfc3339()),
    };
    cred_manager
        .store_platform_tokens(&connection_id, &updated)
        .map_user_err()?;

    // Update last_refreshed_at in DB
    let conn = db.conn.lock().map_user_err()?;
    repository::update_last_refreshed(&conn, &connection_id).map_user_err()?;

    info!("Refreshed Twitch tokens for connection {connection_id}");
    Ok(())
}

/// Revoke Twitch authorization and clean up stored tokens.
#[tauri::command]
pub async fn revoke_twitch_auth(
    connection_id: String,
    db: State<'_, Arc<Database>>,
    cred_manager: State<'_, CredentialManager>,
) -> Result<(), String> {
    // Get existing tokens
    if let Ok(Some(tokens)) = cred_manager.get_platform_tokens(&connection_id) {
        // Best-effort revocation — don't fail the whole operation if Twitch is unreachable
        if let Err(e) = oauth::revoke_token(&tokens.access_token).await {
            error!("Failed to revoke token with Twitch (continuing anyway): {e}");
        }
    }

    // Delete tokens from credential store
    let kind = CredentialKind::PlatformToken {
        connection_id: connection_id.clone(),
    };
    cred_manager.delete_credential(&kind).map_user_err()?;

    // Update status in DB
    let conn = db.conn.lock().map_user_err()?;
    repository::update_connection_status(&conn, &connection_id, "revoked").map_user_err()?;

    info!("Revoked Twitch auth for connection {connection_id}");
    Ok(())
}

/// Generate a cryptographic random state string for CSRF protection.
fn generate_state() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

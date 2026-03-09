use serde::{Deserialize, Serialize};

/// Identifies what kind of credential is being stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CredentialKind {
    /// OAuth tokens for a platform connection (Twitch, YouTube, Kick).
    PlatformToken {
        #[serde(rename = "connectionId")]
        connection_id: String,
    },
    /// API key for an external service (e.g. ElevenLabs, OpenAI).
    ApiKey { service: String },
    /// Internal application secret (e.g. encryption keys, internal tokens).
    AppSecret { key: String },
}

/// OAuth token bundle stored as a single credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthTokens {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// ISO 8601 timestamp when the access token expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_expires_at: Option<String>,
}

/// Which storage backend is active for this session.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CredentialBackend {
    Keychain,
    EncryptedSqlite,
}

/// Generates the storage key string for a given credential kind.
///
/// Format examples:
/// - `contentjuiceos:platform_token:550e8400-e29b-41d4-a716-446655440000`
/// - `contentjuiceos:api_key:elevenlabs`
/// - `contentjuiceos:app_secret:internal_signing_key`
pub fn credential_key(kind: &CredentialKind) -> String {
    match kind {
        CredentialKind::PlatformToken { connection_id } => {
            format!("contentjuiceos:platform_token:{connection_id}")
        }
        CredentialKind::ApiKey { service } => {
            format!("contentjuiceos:api_key:{service}")
        }
        CredentialKind::AppSecret { key } => {
            format!("contentjuiceos:app_secret:{key}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_key_platform_token() {
        let kind = CredentialKind::PlatformToken {
            connection_id: "abc-123".to_string(),
        };
        assert_eq!(
            credential_key(&kind),
            "contentjuiceos:platform_token:abc-123"
        );
    }

    #[test]
    fn credential_key_api_key() {
        let kind = CredentialKind::ApiKey {
            service: "elevenlabs".to_string(),
        };
        assert_eq!(credential_key(&kind), "contentjuiceos:api_key:elevenlabs");
    }

    #[test]
    fn credential_key_app_secret() {
        let kind = CredentialKind::AppSecret {
            key: "signing_key".to_string(),
        };
        assert_eq!(
            credential_key(&kind),
            "contentjuiceos:app_secret:signing_key"
        );
    }

    #[test]
    fn oauth_tokens_serialization_round_trip() {
        let tokens = OAuthTokens {
            access_token: "access123".to_string(),
            refresh_token: Some("refresh456".to_string()),
            token_expires_at: Some("2026-01-01T00:00:00Z".to_string()),
        };
        let json = serde_json::to_string(&tokens).unwrap();
        let deserialized: OAuthTokens = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.access_token, "access123");
        assert_eq!(deserialized.refresh_token.as_deref(), Some("refresh456"));
    }

    #[test]
    fn oauth_tokens_omits_none_fields() {
        let tokens = OAuthTokens {
            access_token: "access123".to_string(),
            refresh_token: None,
            token_expires_at: None,
        };
        let json = serde_json::to_string(&tokens).unwrap();
        assert!(!json.contains("refreshToken"));
        assert!(!json.contains("tokenExpiresAt"));
    }
}

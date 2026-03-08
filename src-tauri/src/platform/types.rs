use serde::{Deserialize, Serialize};

/// A platform connection record matching the `platform_connections` DB table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformConnection {
    pub id: String,
    pub platform: String,
    pub platform_user_id: String,
    pub platform_username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub scopes: String,
    pub status: String,
    pub connected_at: Option<String>,
    pub last_refreshed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Data needed to create or update a platform connection after successful OAuth.
pub struct NewPlatformConnection {
    pub platform: String,
    pub platform_user_id: String,
    pub platform_username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub scopes: Vec<String>,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheType {
    ChannelInfo,
    Emotes,
    Badges,
    Categories,
}

impl CacheType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CacheType::ChannelInfo => "channel_info",
            CacheType::Emotes => "emotes",
            CacheType::Badges => "badges",
            CacheType::Categories => "categories",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "channel_info" => Some(CacheType::ChannelInfo),
            "emotes" => Some(CacheType::Emotes),
            "badges" => Some(CacheType::Badges),
            "categories" => Some(CacheType::Categories),
            _ => None,
        }
    }

    /// Returns the camelCase key used in the `cacheTtl.*` settings,
    /// matching the serde rename on `CacheTtlSettings`.
    pub fn settings_key(&self) -> &'static str {
        match self {
            CacheType::ChannelInfo => "channelInfo",
            CacheType::Emotes => "emotes",
            CacheType::Badges => "badges",
            CacheType::Categories => "categories",
        }
    }

    pub fn all() -> &'static [CacheType] {
        &[
            CacheType::ChannelInfo,
            CacheType::Emotes,
            CacheType::Badges,
            CacheType::Categories,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheEntry {
    pub id: String,
    pub cache_type: String,
    pub cache_key: String,
    pub data: serde_json::Value,
    pub platform: Option<String>,
    pub expires_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheInvalidation {
    pub cache_type: CacheType,
    pub cache_key: Option<String>,
    pub platform: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheStats {
    pub total_entries: u32,
    pub expired_entries: u32,
    pub entries_by_type: Vec<CacheTypeCount>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheTypeCount {
    pub cache_type: String,
    pub count: u32,
}

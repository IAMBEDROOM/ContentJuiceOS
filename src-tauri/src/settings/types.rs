use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GeneralSettings {
    pub language: String,
    pub launch_on_startup: bool,
    pub minimize_to_tray: bool,
    pub check_for_updates: bool,
    pub media_directory: String,
    pub backup_interval_hours: u32,
    pub max_backups: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceSettings {
    pub ui_scale: f64,
    pub show_platform_icons: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ServerSettings {
    pub http_port: u16,
    pub socket_io_port: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ObsSettings {
    pub host: String,
    pub port: u16,
    pub password: String,
    pub auto_connect: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AlertQueueSettings {
    pub mode: String,
    pub delay_between: u32,
    pub max_queue_length: u32,
    pub stale_threshold: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CacheTtlSettings {
    pub channel_info: u32,
    pub emotes: u32,
    pub badges: u32,
    pub categories: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub general: GeneralSettings,
    pub appearance: AppearanceSettings,
    pub server: ServerSettings,
    pub obs: ObsSettings,
    pub alert_queue: AlertQueueSettings,
    pub cache_ttl: CacheTtlSettings,
}

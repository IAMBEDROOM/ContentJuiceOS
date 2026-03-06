use rusqlite::Connection;

use crate::db::error::DbResult;

use super::types::*;

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            launch_on_startup: false,
            minimize_to_tray: true,
            check_for_updates: true,
            media_directory: String::new(),
            backup_interval_hours: 24,
            max_backups: 7,
        }
    }
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            show_platform_icons: true,
        }
    }
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            http_port: 4848,
            socket_io_port: 4849,
        }
    }
}

impl Default for ObsSettings {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 4455,
            password: String::new(),
            auto_connect: false,
        }
    }
}

impl Default for AlertQueueSettings {
    fn default() -> Self {
        Self {
            mode: "sequential".to_string(),
            delay_between: 1000,
            max_queue_length: 50,
            stale_threshold: 300000,
        }
    }
}

impl Default for CacheTtlSettings {
    fn default() -> Self {
        Self {
            channel_info: 300,
            emotes: 3600,
            badges: 3600,
            categories: 600,
        }
    }
}

/// Seeds default settings into the database if the settings table is empty.
/// Runs in a single transaction to ensure atomicity.
pub fn seed_defaults_if_empty(conn: &Connection) -> DbResult<()> {
    let count: u32 = conn.query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))?;

    if count > 0 {
        return Ok(());
    }

    let config = AppConfig::default();
    let sections: &[(&str, serde_json::Value)] = &[
        ("general", serde_json::to_value(&config.general)?),
        ("appearance", serde_json::to_value(&config.appearance)?),
        ("server", serde_json::to_value(&config.server)?),
        ("obs", serde_json::to_value(&config.obs)?),
        ("alertQueue", serde_json::to_value(&config.alert_queue)?),
        ("cacheTtl", serde_json::to_value(&config.cache_ttl)?),
    ];

    let tx = conn.unchecked_transaction()?;

    for (section, value) in sections {
        if let serde_json::Value::Object(map) = value {
            for (field, val) in map {
                let key = format!("{section}.{field}");
                let val_str = match val {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                tx.execute(
                    "INSERT INTO settings (key, value) VALUES (?1, ?2)",
                    rusqlite::params![key, val_str],
                )?;
            }
        }
    }

    tx.commit()?;

    Ok(())
}

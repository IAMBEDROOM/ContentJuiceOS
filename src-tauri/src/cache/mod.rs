pub mod commands;
pub mod error;
pub mod repository;
pub mod types;

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use serde::de::DeserializeOwned;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::db::Database;
use crate::settings::types::CacheTtlSettings;

use error::{CacheError, CacheResult};
use types::{CacheInvalidation, CacheStats, CacheType};

pub struct CacheService {
    db: Arc<Database>,
    invalidation_tx: broadcast::Sender<CacheInvalidation>,
}

impl CacheService {
    pub fn new(db: Arc<Database>) -> Self {
        let (invalidation_tx, _) = broadcast::channel(64);
        Self {
            db,
            invalidation_tx,
        }
    }

    /// Get a cached value, deserializing from JSON into `T`.
    /// Returns `Ok(None)` on miss or expired entry.
    #[allow(dead_code)]
    pub fn get<T: DeserializeOwned>(
        &self,
        cache_type: CacheType,
        key: &str,
        platform: Option<&str>,
    ) -> CacheResult<Option<T>> {
        let conn = self.db.conn.lock().map_err(|e| {
            CacheError::Database(crate::db::error::DbError::Settings(format!(
                "Failed to lock database: {e}"
            )))
        })?;

        let entry = repository::get_entry(&conn, cache_type.as_str(), key, platform)?;

        match entry {
            Some(e) => {
                let value: T = serde_json::from_value(e.data)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Get raw JSON value from cache without deserialization.
    pub fn get_raw(
        &self,
        cache_type: CacheType,
        key: &str,
        platform: Option<&str>,
    ) -> CacheResult<Option<serde_json::Value>> {
        let conn = self.db.conn.lock().map_err(|e| {
            CacheError::Database(crate::db::error::DbError::Settings(format!(
                "Failed to lock database: {e}"
            )))
        })?;

        let entry = repository::get_entry(&conn, cache_type.as_str(), key, platform)?;
        Ok(entry.map(|e| e.data))
    }

    /// Store a value in cache with TTL read from settings.
    #[allow(dead_code)]
    pub fn set(
        &self,
        cache_type: CacheType,
        key: &str,
        platform: Option<&str>,
        data: &serde_json::Value,
    ) -> CacheResult<()> {
        let ttl = self.resolve_ttl(cache_type);
        self.set_with_ttl(cache_type, key, platform, data, ttl)
    }

    /// Store a value in cache with an explicit TTL override (in seconds).
    pub fn set_with_ttl(
        &self,
        cache_type: CacheType,
        key: &str,
        platform: Option<&str>,
        data: &serde_json::Value,
        ttl_secs: u32,
    ) -> CacheResult<()> {
        let conn = self.db.conn.lock().map_err(|e| {
            CacheError::Database(crate::db::error::DbError::Settings(format!(
                "Failed to lock database: {e}"
            )))
        })?;

        let id = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + chrono::Duration::seconds(i64::from(ttl_secs));
        let expires_at_str = expires_at.format("%Y-%m-%d %H:%M:%S").to_string();
        let data_str = serde_json::to_string(data)?;

        repository::upsert_entry(
            &conn,
            &id,
            cache_type.as_str(),
            key,
            platform,
            &data_str,
            &expires_at_str,
        )?;

        Ok(())
    }

    /// Invalidate (delete) a specific cache entry and broadcast the event.
    pub fn invalidate(
        &self,
        cache_type: CacheType,
        key: &str,
        platform: Option<&str>,
    ) -> CacheResult<()> {
        let conn = self.db.conn.lock().map_err(|e| {
            CacheError::Database(crate::db::error::DbError::Settings(format!(
                "Failed to lock database: {e}"
            )))
        })?;

        repository::delete_entry(&conn, cache_type.as_str(), key, platform)?;

        // Broadcast invalidation — ignore errors (no subscribers is fine)
        let _ = self.invalidation_tx.send(CacheInvalidation {
            cache_type,
            cache_key: Some(key.to_string()),
            platform: platform.map(|s| s.to_string()),
        });

        Ok(())
    }

    /// Invalidate all entries of a given type (optionally filtered by platform).
    pub fn invalidate_type(
        &self,
        cache_type: CacheType,
        platform: Option<&str>,
    ) -> CacheResult<()> {
        let conn = self.db.conn.lock().map_err(|e| {
            CacheError::Database(crate::db::error::DbError::Settings(format!(
                "Failed to lock database: {e}"
            )))
        })?;

        repository::delete_by_type(&conn, cache_type.as_str(), platform)?;

        let _ = self.invalidation_tx.send(CacheInvalidation {
            cache_type,
            cache_key: None,
            platform: platform.map(|s| s.to_string()),
        });

        Ok(())
    }

    /// Invalidate all cache entries for a platform across all types.
    #[allow(dead_code)]
    pub fn invalidate_platform(&self, platform: &str) -> CacheResult<()> {
        let conn = self.db.conn.lock().map_err(|e| {
            CacheError::Database(crate::db::error::DbError::Settings(format!(
                "Failed to lock database: {e}"
            )))
        })?;

        repository::delete_by_platform(&conn, platform)?;

        // Broadcast one event per cache type
        for &ct in CacheType::all() {
            let _ = self.invalidation_tx.send(CacheInvalidation {
                cache_type: ct,
                cache_key: None,
                platform: Some(platform.to_string()),
            });
        }

        Ok(())
    }

    /// Subscribe to cache invalidation events.
    #[allow(dead_code)]
    pub fn subscribe(&self) -> broadcast::Receiver<CacheInvalidation> {
        self.invalidation_tx.subscribe()
    }

    /// Get cache statistics.
    pub fn get_stats(&self) -> CacheResult<CacheStats> {
        let conn = self.db.conn.lock().map_err(|e| {
            CacheError::Database(crate::db::error::DbError::Settings(format!(
                "Failed to lock database: {e}"
            )))
        })?;
        Ok(repository::get_stats(&conn)?)
    }

    /// Spawn a background task that purges expired entries every 60 seconds.
    pub fn start_cleanup_task(self: &Arc<Self>) {
        let service = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Ok(conn) = service.db.conn.lock() {
                    let _ = repository::delete_expired(&conn);
                }
            }
        });
    }

    /// Read the TTL for a cache type from settings, falling back to defaults.
    fn resolve_ttl(&self, cache_type: CacheType) -> u32 {
        let defaults = CacheTtlSettings::default();
        let default_ttl = match cache_type {
            CacheType::ChannelInfo => defaults.channel_info,
            CacheType::Emotes => defaults.emotes,
            CacheType::Badges => defaults.badges,
            CacheType::Categories => defaults.categories,
        };

        let conn = match self.db.conn.lock() {
            Ok(c) => c,
            Err(_) => return default_ttl,
        };

        let key = format!("cacheTtl.{}", cache_type.settings_key());

        let result: Result<String, _> =
            conn.query_row("SELECT value FROM settings WHERE key = ?1", [&key], |row| {
                row.get(0)
            });

        match result {
            Ok(val) => val.parse::<u32>().unwrap_or(default_ttl),
            Err(_) => default_ttl,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::run_migrations;
    use crate::settings::defaults::seed_defaults_if_empty;
    use rusqlite::Connection;
    use std::sync::Mutex;

    fn test_db() -> Arc<Database> {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        seed_defaults_if_empty(&conn).unwrap();
        Arc::new(Database {
            conn: Mutex::new(conn),
        })
    }

    #[test]
    fn get_returns_none_for_missing() {
        let db = test_db();
        let service = CacheService::new(db);
        let result: Option<serde_json::Value> =
            service.get(CacheType::Emotes, "nonexistent", None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn set_then_get_roundtrip() {
        let db = test_db();
        let service = CacheService::new(db);
        let data = serde_json::json!({"emotes": ["Kappa", "PogChamp"]});

        service
            .set_with_ttl(CacheType::Emotes, "global", Some("twitch"), &data, 3600)
            .unwrap();

        let result: serde_json::Value = service
            .get(CacheType::Emotes, "global", Some("twitch"))
            .unwrap()
            .expect("entry should exist");
        assert_eq!(result["emotes"][0], "Kappa");
    }

    #[test]
    fn set_with_zero_ttl_immediately_expires() {
        let db = test_db();
        let service = CacheService::new(db);
        let data = serde_json::json!({"v": 1});

        service
            .set_with_ttl(CacheType::Badges, "global", Some("twitch"), &data, 0)
            .unwrap();

        let result: Option<serde_json::Value> = service
            .get(CacheType::Badges, "global", Some("twitch"))
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn invalidate_removes_entry() {
        let db = test_db();
        let service = CacheService::new(db);
        let data = serde_json::json!({"v": 1});

        service
            .set_with_ttl(CacheType::Emotes, "global", Some("twitch"), &data, 3600)
            .unwrap();

        service
            .invalidate(CacheType::Emotes, "global", Some("twitch"))
            .unwrap();

        let result: Option<serde_json::Value> = service
            .get(CacheType::Emotes, "global", Some("twitch"))
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn subscribe_receives_invalidation() {
        let db = test_db();
        let service = CacheService::new(db);
        let mut rx = service.subscribe();

        let data = serde_json::json!({"v": 1});
        service
            .set_with_ttl(CacheType::Emotes, "global", Some("twitch"), &data, 3600)
            .unwrap();

        service
            .invalidate(CacheType::Emotes, "global", Some("twitch"))
            .unwrap();

        let event = rx.try_recv().unwrap();
        assert_eq!(event.cache_type, CacheType::Emotes);
        assert_eq!(event.cache_key, Some("global".to_string()));
        assert_eq!(event.platform, Some("twitch".to_string()));
    }

    #[test]
    fn set_uses_settings_ttl() {
        let db = test_db();
        let service = CacheService::new(db);
        let data = serde_json::json!({"v": 1});

        // Default channelInfo TTL is 300s, so this should be valid
        service
            .set(CacheType::ChannelInfo, "test", Some("twitch"), &data)
            .unwrap();

        let result: Option<serde_json::Value> = service
            .get(CacheType::ChannelInfo, "test", Some("twitch"))
            .unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn get_raw_returns_json_value() {
        let db = test_db();
        let service = CacheService::new(db);
        let data = serde_json::json!({"key": "value"});

        service
            .set_with_ttl(CacheType::Categories, "all", None, &data, 3600)
            .unwrap();

        let result = service.get_raw(CacheType::Categories, "all", None).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap()["key"], "value");
    }

    #[test]
    fn stats_reflect_entries() {
        let db = test_db();
        let service = CacheService::new(Arc::clone(&db));
        let data = serde_json::json!({});

        service
            .set_with_ttl(CacheType::Emotes, "a", Some("twitch"), &data, 3600)
            .unwrap();
        service
            .set_with_ttl(CacheType::Emotes, "b", Some("twitch"), &data, 3600)
            .unwrap();

        // Insert an already-expired entry directly via repository
        {
            let conn = db.conn.lock().unwrap();
            repository::upsert_entry(
                &conn,
                "expired-id",
                "badges",
                "c",
                Some("twitch"),
                "{}",
                "2000-01-01 00:00:00",
            )
            .unwrap();
        }

        let stats = service.get_stats().unwrap();
        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.expired_entries, 1);
    }
}

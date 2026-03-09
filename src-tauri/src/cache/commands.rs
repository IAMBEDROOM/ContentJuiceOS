use std::sync::Arc;

use serde_json::Value;
use tauri::State;

use super::types::{CacheStats, CacheType};
use super::CacheService;

#[tauri::command]
pub fn cache_get(
    cache_type: String,
    cache_key: String,
    platform: Option<String>,
    cache: State<'_, Arc<CacheService>>,
) -> Result<Option<Value>, String> {
    let ct = CacheType::from_str(&cache_type)
        .ok_or_else(|| format!("Invalid cache type: {cache_type}"))?;

    cache
        .get_raw(ct, &cache_key, platform.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cache_invalidate(
    cache_type: String,
    cache_key: Option<String>,
    platform: Option<String>,
    cache: State<'_, Arc<CacheService>>,
) -> Result<(), String> {
    let ct = CacheType::from_str(&cache_type)
        .ok_or_else(|| format!("Invalid cache type: {cache_type}"))?;

    match cache_key {
        Some(key) => cache
            .invalidate(ct, &key, platform.as_deref())
            .map_err(|e| e.to_string()),
        None => cache
            .invalidate_type(ct, platform.as_deref())
            .map_err(|e| e.to_string()),
    }
}

#[tauri::command]
pub fn cache_stats(cache: State<'_, Arc<CacheService>>) -> Result<CacheStats, String> {
    cache.get_stats().map_err(|e| e.to_string())
}

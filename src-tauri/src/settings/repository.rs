use rusqlite::Connection;
use serde_json::Value;

use crate::db::error::{DbError, DbResult};

const VALID_SECTIONS: &[&str] = &[
    "general",
    "appearance",
    "server",
    "obs",
    "alertQueue",
    "cacheTtl",
];

fn validate_section(section: &str) -> DbResult<()> {
    if VALID_SECTIONS.contains(&section) {
        Ok(())
    } else {
        Err(DbError::Settings(format!("Invalid section: {section}")))
    }
}

fn parse_value(raw: &str) -> Value {
    // Try parsing as JSON (numbers, booleans, etc.), fall back to string
    serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_string()))
}

/// Retrieves all settings for a given section as a JSON object.
pub fn get_section(conn: &Connection, section: &str) -> DbResult<Value> {
    validate_section(section)?;

    let prefix = format!("{section}.");
    let mut stmt = conn.prepare("SELECT key, value FROM settings WHERE key LIKE ?1")?;
    let rows = stmt.query_map([format!("{prefix}%")], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut map = serde_json::Map::new();
    for row in rows {
        let (key, value) = row?;
        if let Some(field) = key.strip_prefix(&prefix) {
            map.insert(field.to_string(), parse_value(&value));
        }
    }

    Ok(Value::Object(map))
}

/// Updates all fields in a section. Each field in `data` is upserted individually.
pub fn set_section(conn: &Connection, section: &str, data: &Value) -> DbResult<()> {
    validate_section(section)?;

    let obj = data
        .as_object()
        .ok_or_else(|| DbError::Settings("Section data must be a JSON object".to_string()))?;

    let tx = conn.unchecked_transaction()?;

    for (field, val) in obj {
        let key = format!("{section}.{field}");
        let val_str = match val {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        tx.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
            rusqlite::params![key, val_str],
        )?;
    }

    tx.commit()?;

    Ok(())
}

/// Retrieves all settings grouped by section as a nested JSON object.
pub fn get_full_config(conn: &Connection) -> DbResult<Value> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut sections: serde_json::Map<String, Value> = serde_json::Map::new();

    for row in rows {
        let (key, value) = row?;
        if let Some((section, field)) = key.split_once('.') {
            let section_map = sections
                .entry(section.to_string())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));
            if let Value::Object(map) = section_map {
                map.insert(field.to_string(), parse_value(&value));
            }
        }
    }

    Ok(Value::Object(sections))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::run_migrations;
    use crate::settings::defaults::seed_defaults_if_empty;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        seed_defaults_if_empty(&conn).unwrap();
        conn
    }

    #[test]
    fn seed_defaults_populates_all_keys() {
        let conn = setup_db();
        let count: u32 = conn
            .query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 23); // 7+2+2+4+4+4
    }

    #[test]
    fn seed_defaults_is_idempotent() {
        let conn = setup_db();
        seed_defaults_if_empty(&conn).unwrap();
        let count: u32 = conn
            .query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 23);
    }

    #[test]
    fn get_section_returns_correct_defaults() {
        let conn = setup_db();
        let general = get_section(&conn, "general").unwrap();
        let obj = general.as_object().unwrap();

        assert_eq!(obj.len(), 7);
        assert_eq!(obj["language"], "en");
        assert_eq!(obj["launchOnStartup"], false);
        assert_eq!(obj["minimizeToTray"], true);
        assert_eq!(obj["checkForUpdates"], true);
        assert_eq!(obj["mediaDirectory"], "");
        assert_eq!(obj["backupIntervalHours"], 24);
        assert_eq!(obj["maxBackups"], 7);
    }

    #[test]
    fn get_section_invalid_section_returns_error() {
        let conn = setup_db();
        let result = get_section(&conn, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn set_section_updates_values() {
        let conn = setup_db();

        let mut data = serde_json::Map::new();
        data.insert("language".to_string(), Value::String("fr".to_string()));
        data.insert("launchOnStartup".to_string(), Value::Bool(true));
        data.insert("minimizeToTray".to_string(), Value::Bool(true));
        data.insert("checkForUpdates".to_string(), Value::Bool(true));
        data.insert("mediaDirectory".to_string(), Value::String(String::new()));
        data.insert("backupIntervalHours".to_string(), Value::Number(24.into()));
        data.insert("maxBackups".to_string(), Value::Number(7.into()));

        set_section(&conn, "general", &Value::Object(data)).unwrap();

        let general = get_section(&conn, "general").unwrap();
        assert_eq!(general["language"], "fr");
        assert_eq!(general["launchOnStartup"], true);
    }

    #[test]
    fn set_section_partial_update() {
        let conn = setup_db();

        let mut data = serde_json::Map::new();
        data.insert("language".to_string(), Value::String("de".to_string()));

        set_section(&conn, "general", &Value::Object(data)).unwrap();

        let general = get_section(&conn, "general").unwrap();
        assert_eq!(general["language"], "de");
        // Other fields should remain unchanged
        assert_eq!(general["maxBackups"], 7);
        assert_eq!(general["minimizeToTray"], true);
    }

    #[test]
    fn set_section_invalid_section_returns_error() {
        let conn = setup_db();
        let data = Value::Object(serde_json::Map::new());
        let result = set_section(&conn, "invalid", &data);
        assert!(result.is_err());
    }

    #[test]
    fn get_full_config_returns_all_sections() {
        let conn = setup_db();
        let config = get_full_config(&conn).unwrap();
        let obj = config.as_object().unwrap();

        assert_eq!(obj.len(), 6);
        assert!(obj.contains_key("general"));
        assert!(obj.contains_key("appearance"));
        assert!(obj.contains_key("server"));
        assert!(obj.contains_key("obs"));
        assert!(obj.contains_key("alertQueue"));
        assert!(obj.contains_key("cacheTtl"));
    }

    #[test]
    fn set_then_get_roundtrip() {
        let conn = setup_db();

        let mut data = serde_json::Map::new();
        data.insert("httpPort".to_string(), Value::Number(9999.into()));
        data.insert("socketIoPort".to_string(), Value::Number(9998.into()));

        set_section(&conn, "server", &Value::Object(data)).unwrap();

        let server = get_section(&conn, "server").unwrap();
        assert_eq!(server["httpPort"], 9999);
        assert_eq!(server["socketIoPort"], 9998);
    }
}

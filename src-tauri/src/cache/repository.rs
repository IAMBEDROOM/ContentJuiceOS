use rusqlite::{params, Connection};

use crate::db::error::DbResult;

use super::types::{CacheEntry, CacheStats, CacheTypeCount};

fn row_to_entry(row: &rusqlite::Row) -> rusqlite::Result<CacheEntry> {
    let data_str: String = row.get("data")?;
    let data: serde_json::Value =
        serde_json::from_str(&data_str).unwrap_or(serde_json::Value::Object(Default::default()));

    Ok(CacheEntry {
        id: row.get("id")?,
        cache_type: row.get("cache_type")?,
        cache_key: row.get("cache_key")?,
        data,
        platform: row.get("platform")?,
        expires_at: row.get("expires_at")?,
        created_at: row.get("created_at")?,
    })
}

/// Upsert a cache entry. Uses DELETE + INSERT in a transaction because
/// SQLite treats NULLs as distinct in unique indexes, so `INSERT OR REPLACE`
/// would not match `(type, key, NULL)` against an existing `(type, key, NULL)`.
pub fn upsert_entry(
    conn: &Connection,
    id: &str,
    cache_type: &str,
    cache_key: &str,
    platform: Option<&str>,
    data: &str,
    expires_at: &str,
) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;

    tx.execute(
        "DELETE FROM cache WHERE cache_type = ?1 AND cache_key = ?2 AND ((?3 IS NULL AND platform IS NULL) OR platform = ?3)",
        params![cache_type, cache_key, platform],
    )?;

    tx.execute(
        "INSERT INTO cache (id, cache_type, cache_key, data, platform, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, cache_type, cache_key, data, platform, expires_at],
    )?;

    tx.commit()?;
    Ok(())
}

/// Get a non-expired cache entry matching type, key, and platform.
pub fn get_entry(
    conn: &Connection,
    cache_type: &str,
    cache_key: &str,
    platform: Option<&str>,
) -> DbResult<Option<CacheEntry>> {
    let result = conn.query_row(
        "SELECT * FROM cache WHERE cache_type = ?1 AND cache_key = ?2 AND ((?3 IS NULL AND platform IS NULL) OR platform = ?3) AND expires_at > datetime('now')",
        params![cache_type, cache_key, platform],
        row_to_entry,
    );

    match result {
        Ok(entry) => Ok(Some(entry)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Delete a specific cache entry. Returns number of rows affected.
pub fn delete_entry(
    conn: &Connection,
    cache_type: &str,
    cache_key: &str,
    platform: Option<&str>,
) -> DbResult<u32> {
    let rows = conn.execute(
        "DELETE FROM cache WHERE cache_type = ?1 AND cache_key = ?2 AND ((?3 IS NULL AND platform IS NULL) OR platform = ?3)",
        params![cache_type, cache_key, platform],
    )?;
    Ok(rows as u32)
}

/// Delete all entries of a given type. If platform is Some, only deletes for that platform.
pub fn delete_by_type(
    conn: &Connection,
    cache_type: &str,
    platform: Option<&str>,
) -> DbResult<u32> {
    let rows = if let Some(p) = platform {
        conn.execute(
            "DELETE FROM cache WHERE cache_type = ?1 AND platform = ?2",
            params![cache_type, p],
        )?
    } else {
        conn.execute(
            "DELETE FROM cache WHERE cache_type = ?1",
            params![cache_type],
        )?
    };
    Ok(rows as u32)
}

/// Delete all cache entries for a given platform across all types.
pub fn delete_by_platform(conn: &Connection, platform: &str) -> DbResult<u32> {
    let rows = conn.execute("DELETE FROM cache WHERE platform = ?1", params![platform])?;
    Ok(rows as u32)
}

/// Delete all expired cache entries. Returns number of rows purged.
pub fn delete_expired(conn: &Connection) -> DbResult<u32> {
    let rows = conn.execute("DELETE FROM cache WHERE expires_at < datetime('now')", [])?;
    Ok(rows as u32)
}

/// Get cache statistics: total entries, expired count, and breakdown by type.
pub fn get_stats(conn: &Connection) -> DbResult<CacheStats> {
    let total_entries: u32 = conn.query_row("SELECT COUNT(*) FROM cache", [], |row| row.get(0))?;

    let expired_entries: u32 = conn.query_row(
        "SELECT COUNT(*) FROM cache WHERE expires_at < datetime('now')",
        [],
        |row| row.get(0),
    )?;

    let mut stmt = conn.prepare(
        "SELECT cache_type, COUNT(*) as count FROM cache GROUP BY cache_type ORDER BY cache_type",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(CacheTypeCount {
            cache_type: row.get(0)?,
            count: row.get(1)?,
        })
    })?;

    let mut entries_by_type = Vec::new();
    for row in rows {
        entries_by_type.push(row?);
    }

    Ok(CacheStats {
        total_entries,
        expired_entries,
        entries_by_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::run_migrations;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn upsert_and_get_roundtrip() {
        let conn = test_conn();
        upsert_entry(
            &conn,
            "id1",
            "emotes",
            "global",
            Some("twitch"),
            r#"{"emotes":["Kappa"]}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();

        let entry = get_entry(&conn, "emotes", "global", Some("twitch"))
            .unwrap()
            .expect("entry should exist");
        assert_eq!(entry.cache_type, "emotes");
        assert_eq!(entry.cache_key, "global");
        assert_eq!(entry.platform, Some("twitch".to_string()));
        assert_eq!(entry.data["emotes"][0], "Kappa");
    }

    #[test]
    fn upsert_overwrites_existing() {
        let conn = test_conn();
        upsert_entry(
            &conn,
            "id1",
            "emotes",
            "global",
            Some("twitch"),
            r#"{"v":1}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();
        upsert_entry(
            &conn,
            "id2",
            "emotes",
            "global",
            Some("twitch"),
            r#"{"v":2}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();

        let entry = get_entry(&conn, "emotes", "global", Some("twitch"))
            .unwrap()
            .unwrap();
        assert_eq!(entry.data["v"], 2);
        assert_eq!(entry.id, "id2");

        // Should be only one row
        let count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM cache WHERE cache_type = 'emotes' AND cache_key = 'global'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn null_platform_upsert() {
        let conn = test_conn();
        upsert_entry(
            &conn,
            "id1",
            "categories",
            "all",
            None,
            r#"{"v":1}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();
        upsert_entry(
            &conn,
            "id2",
            "categories",
            "all",
            None,
            r#"{"v":2}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();

        let entry = get_entry(&conn, "categories", "all", None)
            .unwrap()
            .unwrap();
        assert_eq!(entry.data["v"], 2);

        let count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM cache WHERE cache_type = 'categories' AND cache_key = 'all'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn expired_entry_not_returned() {
        let conn = test_conn();
        upsert_entry(
            &conn,
            "id1",
            "badges",
            "global",
            Some("twitch"),
            r#"{}"#,
            "2000-01-01T00:00:00", // already expired
        )
        .unwrap();

        let entry = get_entry(&conn, "badges", "global", Some("twitch")).unwrap();
        assert!(entry.is_none());
    }

    #[test]
    fn delete_entry_removes_row() {
        let conn = test_conn();
        upsert_entry(
            &conn,
            "id1",
            "emotes",
            "global",
            Some("twitch"),
            r#"{}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();

        let deleted = delete_entry(&conn, "emotes", "global", Some("twitch")).unwrap();
        assert_eq!(deleted, 1);

        let entry = get_entry(&conn, "emotes", "global", Some("twitch")).unwrap();
        assert!(entry.is_none());
    }

    #[test]
    fn delete_by_type_removes_all_of_type() {
        let conn = test_conn();
        upsert_entry(
            &conn,
            "id1",
            "emotes",
            "global",
            Some("twitch"),
            r#"{}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();
        upsert_entry(
            &conn,
            "id2",
            "emotes",
            "channel",
            Some("twitch"),
            r#"{}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();
        upsert_entry(
            &conn,
            "id3",
            "badges",
            "global",
            Some("twitch"),
            r#"{}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();

        let deleted = delete_by_type(&conn, "emotes", None).unwrap();
        assert_eq!(deleted, 2);

        // badges should remain
        let remaining: u32 = conn
            .query_row("SELECT COUNT(*) FROM cache", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining, 1);
    }

    #[test]
    fn delete_by_type_with_platform_filter() {
        let conn = test_conn();
        upsert_entry(
            &conn,
            "id1",
            "emotes",
            "global",
            Some("twitch"),
            r#"{}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();
        upsert_entry(
            &conn,
            "id2",
            "emotes",
            "global",
            Some("youtube"),
            r#"{}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();

        let deleted = delete_by_type(&conn, "emotes", Some("twitch")).unwrap();
        assert_eq!(deleted, 1);

        // youtube entry should remain
        let entry = get_entry(&conn, "emotes", "global", Some("youtube")).unwrap();
        assert!(entry.is_some());
    }

    #[test]
    fn delete_by_platform_removes_all_for_platform() {
        let conn = test_conn();
        upsert_entry(
            &conn,
            "id1",
            "emotes",
            "global",
            Some("twitch"),
            r#"{}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();
        upsert_entry(
            &conn,
            "id2",
            "badges",
            "global",
            Some("twitch"),
            r#"{}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();
        upsert_entry(
            &conn,
            "id3",
            "emotes",
            "global",
            Some("youtube"),
            r#"{}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();

        let deleted = delete_by_platform(&conn, "twitch").unwrap();
        assert_eq!(deleted, 2);

        let remaining: u32 = conn
            .query_row("SELECT COUNT(*) FROM cache", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining, 1);
    }

    #[test]
    fn delete_expired_purges_old_entries() {
        let conn = test_conn();
        upsert_entry(
            &conn,
            "id1",
            "emotes",
            "a",
            Some("twitch"),
            r#"{}"#,
            "2000-01-01T00:00:00",
        )
        .unwrap();
        upsert_entry(
            &conn,
            "id2",
            "emotes",
            "b",
            Some("twitch"),
            r#"{}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();

        let purged = delete_expired(&conn).unwrap();
        assert_eq!(purged, 1);

        let remaining: u32 = conn
            .query_row("SELECT COUNT(*) FROM cache", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining, 1);
    }

    #[test]
    fn get_stats_returns_correct_counts() {
        let conn = test_conn();
        upsert_entry(
            &conn,
            "id1",
            "emotes",
            "a",
            Some("twitch"),
            r#"{}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();
        upsert_entry(
            &conn,
            "id2",
            "emotes",
            "b",
            Some("twitch"),
            r#"{}"#,
            "2099-01-01T00:00:00",
        )
        .unwrap();
        upsert_entry(
            &conn,
            "id3",
            "badges",
            "a",
            Some("twitch"),
            r#"{}"#,
            "2000-01-01T00:00:00",
        )
        .unwrap();

        let stats = get_stats(&conn).unwrap();
        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.expired_entries, 1);
        assert_eq!(stats.entries_by_type.len(), 2);

        let badges_count = stats
            .entries_by_type
            .iter()
            .find(|c| c.cache_type == "badges")
            .unwrap();
        assert_eq!(badges_count.count, 1);

        let emotes_count = stats
            .entries_by_type
            .iter()
            .find(|c| c.cache_type == "emotes")
            .unwrap();
        assert_eq!(emotes_count.count, 2);
    }
}

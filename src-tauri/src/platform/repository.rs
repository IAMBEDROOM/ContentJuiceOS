use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::db::error::{DbError, DbResult};

use super::types::{NewPlatformConnection, PlatformConnection};

fn row_to_connection(row: &rusqlite::Row) -> rusqlite::Result<PlatformConnection> {
    Ok(PlatformConnection {
        id: row.get("id")?,
        platform: row.get("platform")?,
        platform_user_id: row.get("platform_user_id")?,
        platform_username: row.get("platform_username")?,
        display_name: row.get("display_name")?,
        avatar_url: row.get("avatar_url")?,
        scopes: row.get("scopes")?,
        status: row.get("status")?,
        connected_at: row.get("connected_at")?,
        last_refreshed_at: row.get("last_refreshed_at")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

/// Insert or update a platform connection. On conflict (same platform + platform_user_id),
/// updates the existing row rather than creating a duplicate.
pub fn upsert_connection(
    conn: &Connection,
    new: &NewPlatformConnection,
) -> DbResult<PlatformConnection> {
    let id = Uuid::new_v4().to_string();
    let scopes_json = serde_json::to_string(&new.scopes)
        .map_err(|e| DbError::Settings(format!("Failed to serialize scopes: {e}")))?;

    conn.execute(
        "INSERT INTO platform_connections (id, platform, platform_user_id, platform_username, display_name, avatar_url, scopes, status, connected_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'connected', datetime('now'), datetime('now'))
         ON CONFLICT(platform, platform_user_id) DO UPDATE SET
             platform_username = excluded.platform_username,
             display_name = excluded.display_name,
             avatar_url = excluded.avatar_url,
             scopes = excluded.scopes,
             status = 'connected',
             connected_at = datetime('now'),
             updated_at = datetime('now')",
        params![
            id,
            new.platform,
            new.platform_user_id,
            new.platform_username,
            new.display_name,
            new.avatar_url,
            scopes_json,
        ],
    )?;

    // Return the connection (may have used existing id on conflict)
    get_connection_by_platform_user(conn, &new.platform, &new.platform_user_id)?
        .ok_or_else(|| DbError::Settings("Upserted connection not found".to_string()))
}

pub fn get_connection(conn: &Connection, id: &str) -> DbResult<Option<PlatformConnection>> {
    let result = conn.query_row(
        "SELECT * FROM platform_connections WHERE id = ?1",
        [id],
        row_to_connection,
    );
    match result {
        Ok(c) => Ok(Some(c)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[allow(dead_code)]
pub fn get_connection_by_platform(
    conn: &Connection,
    platform: &str,
) -> DbResult<Option<PlatformConnection>> {
    let result = conn.query_row(
        "SELECT * FROM platform_connections WHERE platform = ?1 AND status = 'connected' ORDER BY updated_at DESC LIMIT 1",
        [platform],
        row_to_connection,
    );
    match result {
        Ok(c) => Ok(Some(c)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn get_connection_by_platform_user(
    conn: &Connection,
    platform: &str,
    platform_user_id: &str,
) -> DbResult<Option<PlatformConnection>> {
    let result = conn.query_row(
        "SELECT * FROM platform_connections WHERE platform = ?1 AND platform_user_id = ?2",
        params![platform, platform_user_id],
        row_to_connection,
    );
    match result {
        Ok(c) => Ok(Some(c)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn list_connections(conn: &Connection) -> DbResult<Vec<PlatformConnection>> {
    let mut stmt = conn.prepare("SELECT * FROM platform_connections ORDER BY updated_at DESC")?;
    let rows = stmt.query_map([], row_to_connection)?;
    let mut connections = Vec::new();
    for row in rows {
        connections.push(row?);
    }
    Ok(connections)
}

pub fn update_connection_status(conn: &Connection, id: &str, status: &str) -> DbResult<()> {
    conn.execute(
        "UPDATE platform_connections SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![status, id],
    )?;
    Ok(())
}

pub fn update_last_refreshed(conn: &Connection, id: &str) -> DbResult<()> {
    conn.execute(
        "UPDATE platform_connections SET last_refreshed_at = datetime('now'), updated_at = datetime('now') WHERE id = ?1",
        [id],
    )?;
    Ok(())
}

#[allow(dead_code)]
pub fn delete_connection(conn: &Connection, id: &str) -> DbResult<()> {
    conn.execute("DELETE FROM platform_connections WHERE id = ?1", [id])?;
    Ok(())
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

    fn sample_new() -> NewPlatformConnection {
        NewPlatformConnection {
            platform: "twitch".to_string(),
            platform_user_id: "12345".to_string(),
            platform_username: "testuser".to_string(),
            display_name: "TestUser".to_string(),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            scopes: vec!["user:read:email".to_string(), "bits:read".to_string()],
        }
    }

    #[test]
    fn upsert_creates_new_connection() {
        let conn = test_conn();
        let result = upsert_connection(&conn, &sample_new()).unwrap();
        assert_eq!(result.platform, "twitch");
        assert_eq!(result.platform_user_id, "12345");
        assert_eq!(result.status, "connected");
    }

    #[test]
    fn upsert_updates_existing_connection() {
        let conn = test_conn();
        let first = upsert_connection(&conn, &sample_new()).unwrap();

        let mut updated = sample_new();
        updated.display_name = "UpdatedName".to_string();
        let second = upsert_connection(&conn, &updated).unwrap();

        assert_eq!(first.id, second.id);
        assert_eq!(second.display_name, "UpdatedName");
    }

    #[test]
    fn get_connection_by_id() {
        let conn = test_conn();
        let created = upsert_connection(&conn, &sample_new()).unwrap();
        let found = get_connection(&conn, &created.id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, created.id);
    }

    #[test]
    fn get_connection_not_found() {
        let conn = test_conn();
        let found = get_connection(&conn, "nonexistent").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn list_connections_returns_all() {
        let conn = test_conn();
        upsert_connection(&conn, &sample_new()).unwrap();
        let list = list_connections(&conn).unwrap();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn update_status_works() {
        let conn = test_conn();
        let created = upsert_connection(&conn, &sample_new()).unwrap();
        update_connection_status(&conn, &created.id, "disconnected").unwrap();

        let found = get_connection(&conn, &created.id).unwrap().unwrap();
        assert_eq!(found.status, "disconnected");
    }

    #[test]
    fn delete_connection_removes_row() {
        let conn = test_conn();
        let created = upsert_connection(&conn, &sample_new()).unwrap();
        delete_connection(&conn, &created.id).unwrap();

        let found = get_connection(&conn, &created.id).unwrap();
        assert!(found.is_none());
    }
}

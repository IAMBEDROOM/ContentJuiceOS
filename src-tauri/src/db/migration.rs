use log::info;
use rusqlite::Connection;

use super::error::{DbError, DbResult};

pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

const MIGRATION_V1_CORE_SCHEMA: &str = "
-- Settings: key-value store with dot-notation grouping
CREATE TABLE settings (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Platform connections: OAuth connections to streaming platforms
CREATE TABLE platform_connections (
    id                TEXT PRIMARY KEY,
    platform          TEXT NOT NULL CHECK (platform IN ('twitch', 'youtube', 'kick')),
    platform_user_id  TEXT NOT NULL,
    platform_username TEXT NOT NULL,
    display_name      TEXT NOT NULL,
    avatar_url        TEXT,
    scopes            TEXT NOT NULL DEFAULT '[]',
    status            TEXT NOT NULL CHECK (status IN ('connected', 'disconnected', 'expired', 'revoked')) DEFAULT 'disconnected',
    connected_at      TEXT,
    last_refreshed_at TEXT,
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_platform_connections_platform ON platform_connections(platform);

-- Assets: media files (images, audio, video, fonts, animations, captions)
CREATE TABLE assets (
    id                TEXT PRIMARY KEY,
    original_filename TEXT NOT NULL,
    type              TEXT NOT NULL CHECK (type IN ('image', 'audio', 'video', 'font', 'animation', 'caption')),
    format            TEXT NOT NULL,
    file_size         INTEGER NOT NULL,
    width             INTEGER,
    height            INTEGER,
    duration          REAL,
    tags              TEXT NOT NULL DEFAULT '[]',
    file_path         TEXT NOT NULL,
    created_at        TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_assets_type ON assets(type);

-- Designs: visual designs (alerts, overlays, scenes, stingers, panels)
CREATE TABLE designs (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    type       TEXT NOT NULL CHECK (type IN ('alert', 'overlay', 'scene', 'stinger', 'panel')),
    config     TEXT NOT NULL DEFAULT '{}',
    thumbnail  TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_designs_type ON designs(type);

-- Projects: video editor projects
CREATE TABLE projects (
    id                   TEXT PRIMARY KEY,
    name                 TEXT NOT NULL,
    source_video_asset_id TEXT NOT NULL REFERENCES assets(id) ON DELETE RESTRICT,
    config               TEXT NOT NULL DEFAULT '{}',
    thumbnail            TEXT,
    status               TEXT NOT NULL CHECK (status IN ('draft', 'exported')) DEFAULT 'draft',
    created_at           TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at           TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_projects_source_video_asset_id ON projects(source_video_asset_id);
CREATE INDEX idx_projects_status ON projects(status);

-- Voice profiles: voice cloning profiles
CREATE TABLE voice_profiles (
    id                TEXT PRIMARY KEY,
    name              TEXT NOT NULL,
    provider          TEXT NOT NULL CHECK (provider IN ('elevenlabs', 'local', 'system')),
    provider_voice_id TEXT,
    sample_asset_ids  TEXT NOT NULL DEFAULT '[]',
    status            TEXT NOT NULL CHECK (status IN ('pending', 'processing', 'ready', 'error')) DEFAULT 'pending',
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Cache: platform data cache with TTLs
CREATE TABLE cache (
    id         TEXT PRIMARY KEY,
    cache_type TEXT NOT NULL,
    cache_key  TEXT NOT NULL,
    data       TEXT NOT NULL DEFAULT '{}',
    platform   TEXT CHECK (platform IS NULL OR platform IN ('twitch', 'youtube', 'kick')),
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_cache_expires_at ON cache(expires_at);
CREATE INDEX idx_cache_type_key ON cache(cache_type, cache_key);
CREATE UNIQUE INDEX idx_cache_type_key_platform ON cache(cache_type, cache_key, platform);
";

const MIGRATION_V2_SECURE_CREDENTIALS: &str = "
-- Encrypted credential fallback storage (only populated when OS keychain is unavailable)
CREATE TABLE IF NOT EXISTS secure_credentials (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

const MIGRATION_V3_PLATFORM_UNIQUE: &str = "
-- Enforce one connection per platform+user to enable upsert logic
CREATE UNIQUE INDEX IF NOT EXISTS idx_platform_connections_platform_user
ON platform_connections(platform, platform_user_id);
";

const MIGRATION_V4_AUDIT_LOG: &str = "
-- Audit log for security-sensitive operations
CREATE TABLE IF NOT EXISTS audit_log (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp  TEXT NOT NULL DEFAULT (datetime('now')),
    event_type TEXT NOT NULL,
    platform   TEXT,
    details    TEXT NOT NULL DEFAULT '',
    success    INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_log_event_type ON audit_log(event_type);
";

/// Returns all migrations in version order. New migrations are appended here by future tasks.
pub fn all_migrations() -> &'static [Migration] {
    &[
        Migration {
            version: 1,
            name: "core_schema",
            sql: MIGRATION_V1_CORE_SCHEMA,
        },
        Migration {
            version: 2,
            name: "secure_credentials",
            sql: MIGRATION_V2_SECURE_CREDENTIALS,
        },
        Migration {
            version: 3,
            name: "platform_unique_index",
            sql: MIGRATION_V3_PLATFORM_UNIQUE,
        },
        Migration {
            version: 4,
            name: "audit_log",
            sql: MIGRATION_V4_AUDIT_LOG,
        },
    ]
}

/// Runs all pending migrations against the given connection.
///
/// Creates the `_migrations` tracking table if it doesn't exist, then applies
/// any migrations whose version is greater than the current max applied version.
/// Each migration runs in its own transaction so partial progress is preserved.
pub fn run_migrations(conn: &Connection) -> DbResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id         INTEGER PRIMARY KEY,
            version    INTEGER NOT NULL UNIQUE,
            name       TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|e| DbError::Migration(format!("Failed to create _migrations table: {e}")))?;

    let current_version: u32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _migrations",
            [],
            |row| row.get(0),
        )
        .map_err(|e| DbError::Migration(format!("Failed to query migration version: {e}")))?;

    let pending: Vec<&Migration> = all_migrations()
        .iter()
        .filter(|m| m.version > current_version)
        .collect();

    if pending.is_empty() {
        info!("Database is up to date (version {current_version})");
        return Ok(());
    }

    // Verify migrations are strictly ascending
    for window in pending.windows(2) {
        if window[0].version >= window[1].version {
            return Err(DbError::Migration(format!(
                "Migration versions are not strictly ascending: {} >= {}",
                window[0].version, window[1].version
            )));
        }
    }

    for migration in &pending {
        info!(
            "Applying migration v{}: {}",
            migration.version, migration.name
        );

        let tx = conn
            .unchecked_transaction()
            .map_err(|e| DbError::Migration(format!("Failed to begin transaction: {e}")))?;

        tx.execute_batch(migration.sql).map_err(|e| {
            DbError::Migration(format!(
                "Migration v{} '{}' failed: {e}",
                migration.version, migration.name
            ))
        })?;

        tx.execute(
            "INSERT INTO _migrations (version, name) VALUES (?1, ?2)",
            rusqlite::params![migration.version, migration.name],
        )
        .map_err(|e| {
            DbError::Migration(format!(
                "Failed to record migration v{}: {e}",
                migration.version
            ))
        })?;

        tx.commit().map_err(|e| {
            DbError::Migration(format!(
                "Failed to commit migration v{}: {e}",
                migration.version
            ))
        })?;
    }

    info!(
        "Applied {} migration(s), now at version {}",
        pending.len(),
        pending.last().unwrap().version
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn creates_migrations_table_on_empty_db() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let count: u32 = conn
            .query_row("SELECT COUNT(*) FROM _migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn running_migrations_twice_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        let count: u32 = conn
            .query_row("SELECT COUNT(*) FROM _migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn migrations_apply_in_order() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let max_version: u32 = conn
            .query_row("SELECT MAX(version) FROM _migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(max_version, 4);

        // Running again should be a no-op
        run_migrations(&conn).unwrap();

        let count: u32 = conn
            .query_row("SELECT COUNT(*) FROM _migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn v2_creates_secure_credentials_table() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = 'secure_credentials'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            exists,
            "secure_credentials table should exist after V2 migration"
        );

        // Verify we can insert and query
        conn.execute(
            "INSERT INTO secure_credentials (key, value) VALUES ('test_key', 'test_value')",
            [],
        )
        .unwrap();

        let value: String = conn
            .query_row(
                "SELECT value FROM secure_credentials WHERE key = 'test_key'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(value, "test_value");
    }

    #[test]
    fn v1_creates_all_core_tables() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let expected_tables = [
            "settings",
            "platform_connections",
            "assets",
            "designs",
            "projects",
            "voice_profiles",
            "cache",
        ];

        for table in &expected_tables {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert!(exists, "Table '{}' should exist", table);
        }
    }

    #[test]
    fn v1_settings_table_has_correct_columns() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('general.language', 'en')",
            [],
        )
        .unwrap();

        let value: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'general.language'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(value, "en");

        // updated_at should be auto-populated
        let updated_at: String = conn
            .query_row(
                "SELECT updated_at FROM settings WHERE key = 'general.language'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!updated_at.is_empty());
    }

    #[test]
    fn v1_projects_foreign_key_enforced() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();

        let result = conn.execute(
            "INSERT INTO projects (id, name, source_video_asset_id) VALUES ('p1', 'Test Project', 'nonexistent-asset-id')",
            [],
        );
        assert!(
            result.is_err(),
            "Foreign key constraint should reject nonexistent asset ID"
        );
    }

    #[test]
    fn v1_platform_connections_check_constraint() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let result = conn.execute(
            "INSERT INTO platform_connections (id, platform, platform_user_id, platform_username, display_name) VALUES ('pc1', 'invalid_platform', 'uid', 'uname', 'dname')",
            [],
        );
        assert!(
            result.is_err(),
            "CHECK constraint should reject invalid platform value"
        );
    }

    #[test]
    fn v1_cache_unique_constraint() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        conn.execute(
            "INSERT INTO cache (id, cache_type, cache_key, platform, expires_at) VALUES ('c1', 'user_info', 'key1', 'twitch', datetime('now', '+1 hour'))",
            [],
        )
        .unwrap();

        let result = conn.execute(
            "INSERT INTO cache (id, cache_type, cache_key, platform, expires_at) VALUES ('c2', 'user_info', 'key1', 'twitch', datetime('now', '+1 hour'))",
            [],
        );
        assert!(
            result.is_err(),
            "Unique constraint should reject duplicate (cache_type, cache_key, platform) tuple"
        );
    }
}

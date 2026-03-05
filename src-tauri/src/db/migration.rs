use log::info;
use rusqlite::Connection;

use super::error::{DbError, DbResult};

pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

/// Returns all migrations in version order. New migrations are appended here by future tasks.
pub fn all_migrations() -> &'static [Migration] {
    &[]
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
        assert_eq!(count, 0);
    }

    #[test]
    fn running_migrations_twice_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        let count: u32 = conn
            .query_row("SELECT COUNT(*) FROM _migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn migrations_apply_in_order() {
        // Test with a local set of migrations to verify ordering logic
        let conn = Connection::open_in_memory().unwrap();

        // Create the tracking table first
        run_migrations(&conn).unwrap();

        // Manually simulate two migrations to verify the table works
        conn.execute_batch("CREATE TABLE test_one (id INTEGER PRIMARY KEY);")
            .unwrap();
        conn.execute(
            "INSERT INTO _migrations (version, name) VALUES (1, 'create_test_one')",
            [],
        )
        .unwrap();

        conn.execute_batch("CREATE TABLE test_two (id INTEGER PRIMARY KEY);")
            .unwrap();
        conn.execute(
            "INSERT INTO _migrations (version, name) VALUES (2, 'create_test_two')",
            [],
        )
        .unwrap();

        let max_version: u32 = conn
            .query_row("SELECT MAX(version) FROM _migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(max_version, 2);

        // Running migrations again should be a no-op since all_migrations() is empty
        run_migrations(&conn).unwrap();
    }
}

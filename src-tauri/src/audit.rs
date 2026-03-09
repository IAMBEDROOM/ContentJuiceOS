//! Audit logging for security-sensitive operations.
//!
//! Writes to the `audit_log` SQLite table. Each entry records what happened,
//! which platform (if applicable), whether it succeeded, and a free-text detail.
//! Entries older than 90 days are pruned on startup.

use std::sync::Arc;

use log::{error, info};
use rusqlite::params;

use crate::db::Database;

/// Categories of auditable events.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum AuditEvent {
    AuthStarted,
    AuthCompleted,
    AuthFailed,
    TokenRefreshed,
    TokenRevoked,
    CredentialAccessed,
    BackupCreated,
    BackupRestored,
}

impl AuditEvent {
    fn as_str(self) -> &'static str {
        match self {
            Self::AuthStarted => "auth_started",
            Self::AuthCompleted => "auth_completed",
            Self::AuthFailed => "auth_failed",
            Self::TokenRefreshed => "token_refreshed",
            Self::TokenRevoked => "token_revoked",
            Self::CredentialAccessed => "credential_accessed",
            Self::BackupCreated => "backup_created",
            Self::BackupRestored => "backup_restored",
        }
    }
}

/// Log an audit event. Failures are logged but never propagated — audit logging
/// must not break the operation being audited.
#[allow(dead_code)]
pub fn log_audit(
    db: &Arc<Database>,
    event: AuditEvent,
    platform: Option<&str>,
    details: &str,
    success: bool,
) {
    let Ok(conn) = db.conn.lock() else {
        error!("Audit: could not acquire DB lock for {:?}", event);
        return;
    };

    if let Err(e) = conn.execute(
        "INSERT INTO audit_log (event_type, platform, details, success) VALUES (?1, ?2, ?3, ?4)",
        params![event.as_str(), platform, details, success as i32],
    ) {
        error!("Audit: failed to write {:?}: {e}", event);
    }
}

/// Delete audit entries older than 90 days. Called at app startup.
pub fn prune_old_entries(db: &Arc<Database>) {
    let Ok(conn) = db.conn.lock() else {
        return;
    };

    match conn.execute(
        "DELETE FROM audit_log WHERE timestamp < datetime('now', '-90 days')",
        [],
    ) {
        Ok(count) if count > 0 => info!("Audit: pruned {count} entries older than 90 days"),
        Err(e) => error!("Audit: failed to prune old entries: {e}"),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::sync::Arc;

    fn test_db() -> Arc<Database> {
        let tmp = std::env::temp_dir().join(format!(
            "cjos_audit_test_{}",
            uuid::Uuid::new_v4()
        ));
        let _ = std::fs::create_dir_all(&tmp);
        Arc::new(Database::initialize(&tmp).unwrap())
    }

    #[test]
    fn log_and_query_audit_event() {
        let db = test_db();
        log_audit(&db, AuditEvent::AuthCompleted, Some("twitch"), "user123", true);

        let conn = db.conn.lock().unwrap();
        let (event_type, platform, details, success): (String, String, String, i32) = conn
            .query_row(
                "SELECT event_type, platform, details, success FROM audit_log ORDER BY id DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();

        assert_eq!(event_type, "auth_completed");
        assert_eq!(platform, "twitch");
        assert_eq!(details, "user123");
        assert_eq!(success, 1);
    }

    #[test]
    fn prune_does_not_delete_recent() {
        let db = test_db();
        log_audit(&db, AuditEvent::BackupCreated, None, "test", true);
        prune_old_entries(&db);

        let conn = db.conn.lock().unwrap();
        let count: u32 = conn
            .query_row("SELECT COUNT(*) FROM audit_log", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}

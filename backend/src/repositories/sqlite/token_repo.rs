use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use crate::{
    error::{AppError, Result},
    repositories::traits::TokenRepository,
};

pub struct SqliteTokenRepository {
    db_path: String,
}

impl SqliteTokenRepository {
    pub fn new(db_path: String) -> Self {
        Self { db_path }
    }
}

#[async_trait]
impl TokenRepository for SqliteTokenRepository {
    async fn revoke(&self, jti: &str, expires_at: DateTime<Utc>) -> Result<()> {
        let db_path = self.db_path.clone();
        let jti = jti.to_string();
        let expires_at = expires_at;

        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| AppError::Database(e))?;
            conn.execute(
                "INSERT OR REPLACE INTO revoked_tokens (jti, revoked_at, expires_at) VALUES (?, ?, ?)",
                params![jti, Utc::now(), expires_at],
            )?;
            Ok(())
        })
        .await
        .map_err(|_| AppError::internal())?
    }

    async fn is_revoked(&self, jti: &str) -> Result<bool> {
        let db_path = self.db_path.clone();
        let jti = jti.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| AppError::Database(e))?;
            let mut stmt = conn.prepare("SELECT 1 FROM revoked_tokens WHERE jti = ?")?;
            
            match stmt.exists(params![jti]) {
                Ok(exists) => Ok(exists),
                Err(e) => Err(AppError::Database(e)),
            }
        })
        .await
        .map_err(|_| AppError::internal())?
    }

    async fn cleanup_expired(&self) -> Result<u64> {
        let db_path = self.db_path.clone();

        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| AppError::Database(e))?;
            let rows_deleted = conn.execute(
                "DELETE FROM revoked_tokens WHERE expires_at < ?",
                params![Utc::now()],
            )?;
            Ok(rows_deleted as u64)
        })
        .await
        .map_err(|_| AppError::internal())?
    }
}

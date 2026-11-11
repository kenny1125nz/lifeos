use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use crate::{
    error::{AppError, Result},
    models::{NewUser, User},
    repositories::traits::UserRepository,
};

pub struct SqliteUserRepository {
    db_path: String,
}

impl SqliteUserRepository {
    pub fn new(db_path: String) -> Self {
        Self { db_path }
    }

    fn get_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path).map_err(|e| AppError::Database(e))
    }

    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<User> {
        Ok(User {
            id: row.get(0)?,
            email: row.get(1)?,
            password_hash: row.get(2)?,
            created_at: DateTime::from_utc(row.get::<_, DateTime<Utc>>(3)?.naive_utc(), Utc),
            updated_at: DateTime::from_utc(row.get::<_, DateTime<Utc>>(4)?.naive_utc(), Utc),
            verified: row.get::<_, i64>(5)? != 0,
            locked: row.get::<_, i64>(6)? != 0,
            failed_attempts: row.get::<_, i64>(7)? as u32,
        })
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn create(&self, user: &NewUser, password_hash: &str) -> Result<User> {
        let db_path = self.db_path.clone();
        let user = user.clone();
        let password_hash = password_hash.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| AppError::Database(e))?;
            let now = Utc::now();

            conn.execute(
                "INSERT INTO users (id, email, password, created_at, updated_at, verified, locked, failed_attempts) 
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    uuid::Uuid::new_v4().to_string(),
                    user.email,
                    password_hash,
                    now,
                    now,
                    0, // verified
                    0, // locked
                    0, // failed_attempts
                ],
            )?;

            let id = conn.last_insert_rowid().to_string();
            let mut stmt = conn.prepare("SELECT * FROM users WHERE id = ?")?;
            let user = stmt.query_row(params![id], Self::from_row)?;

            Ok(user)
        })
        .await
        .map_err(|_| AppError::internal())?
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<User>> {
        let db_path = self.db_path.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| AppError::Database(e))?;
            let mut stmt = conn.prepare("SELECT * FROM users WHERE id = ?")?;
            
            match stmt.query_row(params![id], Self::from_row) {
                Ok(user) => Ok(Some(user)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(AppError::Database(e)),
            }
        })
        .await
        .map_err(|_| AppError::internal())?
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let db_path = self.db_path.clone();
        let email = email.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| AppError::Database(e))?;
            let mut stmt = conn.prepare("SELECT * FROM users WHERE email = ?")?;
            
            match stmt.query_row(params![email], Self::from_row) {
                Ok(user) => Ok(Some(user)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(AppError::Database(e)),
            }
        })
        .await
        .map_err(|_| AppError::internal())?
    }

    async fn update(&self, user: &User) -> Result<User> {
        let db_path = self.db_path.clone();
        let user = user.clone();

        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| AppError::Database(e))?;

            conn.execute(
                "UPDATE users SET email = ?, password = ?, updated_at = ?, verified = ?, locked = ?, failed_attempts = ? WHERE id = ?",
                params![
                    user.email,
                    user.password_hash,
                    user.updated_at,
                    user.verified as i64,
                    user.locked as i64,
                    user.failed_attempts as i64,
                    user.id,
                ],
            )?;

            let mut stmt = conn.prepare("SELECT * FROM users WHERE id = ?")?;
            let updated_user = stmt.query_row(params![user.id], Self::from_row)?;

            Ok(updated_user)
        })
        .await
        .map_err(|_| AppError::internal())?
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let db_path = self.db_path.clone();
        let id = id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| AppError::Database(e))?;
            conn.execute("DELETE FROM users WHERE id = ?", params![id])?;
            Ok(())
        })
        .await
        .map_err(|_| AppError::internal())?
    }

    async fn increment_failed_attempts(&self, email: &str) -> Result<()> {
        let db_path = self.db_path.clone();
        let email = email.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| AppError::Database(e))?;
            
            // Get current attempts
            let mut stmt = conn.prepare("SELECT failed_attempts FROM users WHERE email = ?")?;
            let current_attempts: i64 = match stmt.query_row(params![&email], |row| row.get(0)) {
                Ok(val) => val,
                Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(()),
                Err(e) => return Err(AppError::Database(e)),
            };

            let new_attempts = current_attempts + 1;
            let locked = new_attempts >= 5;

            conn.execute(
                "UPDATE users SET failed_attempts = ?, locked = ?, updated_at = ? WHERE email = ?",
                params![new_attempts, locked as i64, Utc::now(), email],
            )?;

            Ok(())
        })
        .await
        .map_err(|_| AppError::internal())?
    }

    async fn reset_failed_attempts(&self, email: &str) -> Result<()> {
        let db_path = self.db_path.clone();
        let email = email.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&db_path).map_err(|e| AppError::Database(e))?;
            conn.execute(
                "UPDATE users SET failed_attempts = 0, locked = 0, updated_at = ? WHERE email = ?",
                params![Utc::now(), email],
            )?;
            Ok(())
        })
        .await
        .map_err(|_| AppError::internal())?
    }
}

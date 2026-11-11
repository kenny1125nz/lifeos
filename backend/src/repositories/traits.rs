use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    error::Result,
    models::{NewUser, User},
};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &NewUser, password_hash: &str) -> Result<User>;
    async fn find_by_id(&self, id: &str) -> Result<Option<User>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn update(&self, user: &User) -> Result<User>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn increment_failed_attempts(&self, email: &str) -> Result<()>;
    async fn reset_failed_attempts(&self, email: &str) -> Result<()>;
}

#[async_trait]
pub trait TokenRepository: Send + Sync {
    async fn revoke(&self, jti: &str, expires_at: DateTime<Utc>) -> Result<()>;
    async fn is_revoked(&self, jti: &str) -> Result<bool>;
    async fn cleanup_expired(&self) -> Result<u64>;
}

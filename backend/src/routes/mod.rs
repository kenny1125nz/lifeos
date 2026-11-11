pub mod auth;
pub mod static_files;

use axum::Router;
use std::sync::Arc;

use crate::{
    repositories::{
        sqlite::{SqliteTokenRepository, SqliteUserRepository},
        traits::{TokenRepository, UserRepository},
    },
    services::{auth::AuthService, token::TokenService},
};

pub fn create_router(
    user_repo: Arc<dyn UserRepository>,
    token_repo: Arc<dyn TokenRepository>,
    token_service: Arc<TokenService>,
    static_dir: String,
) -> Router {
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        token_repo.clone(),
        token_service.clone(),
    ));

    Router::new()
        .nest("/api/auth", auth::create_router(auth_service.clone()))
        .merge(static_files::create_router(static_dir))
        .layer(axum::Extension(user_repo))
        .layer(axum::Extension(token_repo))
        .layer(axum::Extension(token_service))
        .layer(axum::Extension(auth_service))
}

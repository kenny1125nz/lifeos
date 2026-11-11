use std::sync::Arc;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};

use crate::{
    error::Result,
    middleware::auth::{AuthMiddleware, AuthState},
    models::{LoginRequest, NewUser},
    services::auth::AuthService,
};

pub fn create_router(auth_service: Arc<AuthService>) -> Router {
    Router::new()
        .route("/signup", post(signup))
        .route("/signin", post(signin))
        .route("/me", get(get_current_user))
        .route("/refresh", post(refresh_token))
        .with_state(auth_service)
}

async fn signup(
    State(auth_service): State<Arc<AuthService>>,
    Json(new_user): Json<NewUser>,
) -> Result<Json<crate::models::AuthResponse>> {
    let response = auth_service.signup(new_user).await?;
    Ok(Json(response))
}

async fn signin(
    State(auth_service): State<Arc<AuthService>>,
    Json(login_request): Json<LoginRequest>,
) -> Result<Json<crate::models::AuthResponse>> {
    let response = auth_service.signin(login_request).await?;
    Ok(Json(response))
}

async fn get_current_user(
    State(auth_service): State<Arc<AuthService>>,
    AuthMiddleware(auth_state): AuthMiddleware,
) -> Result<Json<crate::models::UserResponse>> {
    let response = auth_service.get_current_user(&auth_state.user_id).await?;
    Ok(Json(response))
}

async fn refresh_token(
    State(auth_service): State<Arc<AuthService>>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<crate::models::AuthResponse>> {
    let response = auth_service.refresh_token(&payload.refresh_token).await?;
    Ok(Json(response))
}

#[derive(serde::Deserialize)]
struct RefreshTokenRequest {
    refresh_token: String,
}

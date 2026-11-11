use std::sync::Arc;

use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::{
    error::AppError,
    services::token::TokenService,
};

#[derive(Clone)]
pub struct AuthState {
    pub user_id: String,
    pub email: String,
    pub jti: String,
}

pub struct AuthMiddleware(pub AuthState);

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for AuthMiddleware
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract token from Authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| {
                AppError::Auth("Missing or invalid authorization header".to_string())
            })?;

        // Get token service from extensions
        let token_service = parts
            .extensions
            .get::<Arc<TokenService>>()
            .ok_or_else(|| AppError::internal())?;

        // Verify token
        let claims = token_service.verify_token(bearer.token()).await?;

        // Get token repository from extensions
        let token_repo = parts
            .extensions
            .get::<Arc<dyn crate::repositories::traits::TokenRepository>>()
            .ok_or_else(|| AppError::internal())?;

        // Check if token is revoked
        if token_repo.is_revoked(&claims.jti).await? {
            return Err(AppError::Token("Token has been revoked".to_string()));
        }

        Ok(AuthMiddleware(AuthState {
            user_id: claims.sub,
            email: claims.email,
            jti: claims.jti,
        }))
    }
}

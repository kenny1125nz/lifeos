use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    models::{TokenClaims, User},
};

pub struct TokenService {
    jwt_secret: String,
    token_expiry: i64,
    refresh_token_expiry: i64,
}

impl TokenService {
    pub fn new(jwt_secret: String, token_expiry: i64, refresh_token_expiry: i64) -> Self {
        Self {
            jwt_secret,
            token_expiry,
            refresh_token_expiry,
        }
    }

    pub async fn generate_tokens(&self, user: &User) -> Result<(String, String)> {
        use std::time::Instant;
        let start_time = Instant::now();
        
        let now = Utc::now();
        let token_jti = Uuid::new_v4().to_string();
        let refresh_jti = Uuid::new_v4().to_string();

        // Access token claims
        let token_claims = TokenClaims {
            sub: user.id.clone(),
            email: user.email.clone(),
            iat: now.timestamp(),
            exp: (now + Duration::seconds(self.token_expiry)).timestamp(),
            jti: token_jti,
        };

        // Refresh token claims
        let refresh_claims = TokenClaims {
            sub: user.id.clone(),
            email: user.email.clone(),
            iat: now.timestamp(),
            exp: (now + Duration::seconds(self.refresh_token_expiry)).timestamp(),
            jti: refresh_jti,
        };

        // Generate tokens
        let encode_start = Instant::now();
        let token = encode(
            &Header::default(),
            &token_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Token(format!("Failed to generate token: {}", e)))?;

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Token(format!("Failed to generate refresh token: {}", e)))?;
        let encode_duration = encode_start.elapsed();

        let total_duration = start_time.elapsed();
        
        // Log performance metrics
        tracing::info!(
            "Token generation completed in {:?} (encoding: {:?}) for user {}",
            total_duration,
            encode_duration,
            user.email
        );

        Ok((token, refresh_token))
    }

    pub async fn verify_token(&self, token: &str) -> Result<TokenClaims> {
        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::Token(format!("Invalid token: {}", e)))?;

        // Check if token is expired
        let now = Utc::now().timestamp();
        if token_data.claims.exp < now {
            return Err(AppError::Token("Token has expired".to_string()));
        }

        Ok(token_data.claims)
    }

    pub async fn verify_refresh_token(&self, token: &str) -> Result<TokenClaims> {
        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::Token(format!("Invalid refresh token: {}", e)))?;

        // Check if token is expired
        let now = Utc::now().timestamp();
        if token_data.claims.exp < now {
            return Err(AppError::Token("Refresh token has expired".to_string()));
        }

        Ok(token_data.claims)
    }

    pub fn get_jwt_secret(&self) -> &str {
        &self.jwt_secret
    }

    pub fn get_token_expiry(&self) -> i64 {
        self.token_expiry
    }

    pub fn get_refresh_token_expiry(&self) -> i64 {
        self.refresh_token_expiry
    }
}

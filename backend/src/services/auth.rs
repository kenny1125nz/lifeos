use std::sync::Arc;

use bcrypt::{hash, verify};
use validator::Validate;

use crate::{
    error::{AppError, Result},
    models::{AuthResponse, LoginRequest, NewUser, UserResponse},
    repositories::traits::{TokenRepository, UserRepository},
    services::token::TokenService,
};

pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    token_repo: Arc<dyn TokenRepository>,
    token_service: Arc<TokenService>,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        token_repo: Arc<dyn TokenRepository>,
        token_service: Arc<TokenService>,
    ) -> Self {
        Self {
            user_repo,
            token_repo,
            token_service,
        }
    }

    pub async fn signup(&self, new_user: NewUser) -> Result<AuthResponse> {
        // Validate input
        new_user.validate().map_err(|e| {
            AppError::Validation(format!("Validation failed: {}", e))
        })?;

        // Check if user already exists
        if let Some(_) = self.user_repo.find_by_email(&new_user.email).await? {
            return Err(AppError::Auth("User already exists".to_string()));
        }

        // Hash password with bcrypt using cost 6 (very fast, suitable for personal projects)
        let password_hash = hash(&new_user.password, 6)
            .map_err(|e| AppError::Auth(format!("Failed to hash password: {}", e)))?;

        // Create user
        let user = self.user_repo.create(&new_user, &password_hash).await?;

        // Generate tokens
        let (token, refresh_token) = self.token_service.generate_tokens(&user).await?;

        Ok(AuthResponse {
            user: UserResponse::from(user),
            token,
            refresh_token,
        })
    }

    pub async fn signin(&self, login_request: LoginRequest) -> Result<AuthResponse> {
        use std::time::Instant;
        let start_time = Instant::now();
        
        // Validate input
        login_request.validate().map_err(|e| {
            AppError::Validation(format!("Validation failed: {}", e))
        })?;

        // Find user
        let find_user_start = Instant::now();
        let user = match self.user_repo.find_by_email(&login_request.email).await? {
            Some(user) => user,
            None => {
                return Err(AppError::Auth("Invalid email or password".to_string()));
            }
        };
        let find_user_duration = find_user_start.elapsed();
        
        // Check if account is locked
        if user.locked {
            return Err(AppError::Auth("Account is locked".to_string()));
        }

        // Verify password with bcrypt
        let verify_start = Instant::now();
        let password_valid = verify(&login_request.password, &user.password_hash)
            .map_err(|e| AppError::Auth(format!("Password verification failed: {}", e)))?;
        let verify_duration = verify_start.elapsed();

        if !password_valid {
            // Increment failed attempts
            self.user_repo
                .increment_failed_attempts(&login_request.email)
                .await?;
            return Err(AppError::Auth("Invalid email or password".to_string()));
        }

        // Reset failed attempts on successful login
        self.user_repo
            .reset_failed_attempts(&login_request.email)
            .await?;

        // Re-hash password with lower cost if it was hashed with higher cost
        // This improves performance for future logins
        if user.password_hash.starts_with("$2b$12$") {
            // Password was hashed with cost 12, re-hash with cost 6 (very fast)
            let new_hash = hash(&login_request.password, 6)
                .map_err(|e| AppError::Auth(format!("Failed to re-hash password: {}", e)))?;
            
            // Update user with new hash
            let mut updated_user = user.clone();
            updated_user.password_hash = new_hash;
            self.user_repo.update(&updated_user).await?;
            
            tracing::info!("Re-hashed password for user {} from cost 12 to cost 6", user.email);
        }

        // Generate tokens
        let token_start = Instant::now();
        let (token, refresh_token) = self.token_service.generate_tokens(&user).await?;
        let token_duration = token_start.elapsed();

        let total_duration = start_time.elapsed();
        
        // Log performance metrics
        tracing::info!(
            "Signin completed in {:?} (find_user: {:?}, verify: {:?}, tokens: {:?})",
            total_duration,
            find_user_duration,
            verify_duration,
            token_duration
        );

        Ok(AuthResponse {
            user: UserResponse::from(user),
            token,
            refresh_token,
        })
    }

    pub async fn get_current_user(&self, user_id: &str) -> Result<UserResponse> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(UserResponse::from(user))
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResponse> {
        // Verify refresh token
        let claims = self.token_service.verify_refresh_token(refresh_token).await?;

        // Check if token is revoked
        if self.token_repo.is_revoked(&claims.jti).await? {
            return Err(AppError::Token("Token has been revoked".to_string()));
        }

        // Get user
        let user = self
            .user_repo
            .find_by_id(&claims.sub)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Generate new tokens
        let (token, refresh_token) = self.token_service.generate_tokens(&user).await?;

        Ok(AuthResponse {
            user: UserResponse::from(user),
            token,
            refresh_token,
        })
    }
}

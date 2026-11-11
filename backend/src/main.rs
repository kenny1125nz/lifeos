mod config;
mod error;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;

use std::{fs, path::Path, sync::Arc};

use axum::Extension;
use config::Config;
use dotenv::dotenv;
use error::Result;
use repositories::{
    sqlite::{SqliteTokenRepository, SqliteUserRepository},
    traits::{TokenRepository, UserRepository},
};
use routes::create_router;
use services::token::TokenService;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize configuration
    let config = Config::from_env().map_err(|e| {
        eprintln!("Failed to load configuration: {}", e);
        std::process::exit(1);
    }).map_err(|_| error::AppError::Internal("Failed to load configuration".to_string()))?;

    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting LifeOS Backend v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Configuration loaded: {:?}", config);

    // Create database directory if it doesn't exist
    if let Some(parent) = Path::new(&config.database_path).parent() {
        fs::create_dir_all(parent).map_err(|e| {
            error::AppError::internal().with_context(|| format!("Failed to create data directory: {}", e))
        })?;
    }

    // Run migrations
    run_migrations(&config.database_path)?;

    // Create repositories
    let user_repo: Arc<dyn UserRepository> = Arc::new(SqliteUserRepository::new(config.database_path.clone()));
    let token_repo: Arc<dyn TokenRepository> = Arc::new(SqliteTokenRepository::new(config.database_path.clone()));

    // Create token service
    let token_service = Arc::new(TokenService::new(
        config.jwt_secret.clone(),
        config.token_expiry,
        config.refresh_token_expiry,
    ));

    // Create router
    let app = create_router(
        user_repo,
        token_repo,
        token_service,
        config.static_dir.clone(),
    );

    // Start server
    let listener = tokio::net::TcpListener::bind(config.bind_address())
        .await
        .map_err(|e| {
            error::AppError::internal().with_context(|| format!("Failed to bind to address: {}", e))
        })?;

    tracing::info!("Server listening on {}", config.bind_address());
    tracing::info!("Static files directory: {}", config.static_dir);
    tracing::info!("Database path: {}", config.database_path);

    axum::serve(listener, app)
        .await
        .map_err(|e| error::AppError::internal().with_context(|| format!("Server error: {}", e)))?;

    Ok(())
}

fn run_migrations(db_path: &str) -> Result<()> {
    use rusqlite::Connection;
    
    let conn = Connection::open(db_path).map_err(|e| error::AppError::Database(e))?;
    
    // Read and execute migration SQL
    let migration_sql = include_str!("../migrations/init.sql");
    conn.execute_batch(migration_sql)
        .map_err(|e| error::AppError::Database(e))?;

    tracing::info!("Database migrations applied successfully");
    Ok(())
}

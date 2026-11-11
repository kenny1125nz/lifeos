use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_path: String,
    pub jwt_secret: String,
    pub token_expiry: i64,
    pub refresh_token_expiry: i64,
    pub host: String,
    pub port: u16,
    pub static_dir: String,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            database_path: env::var("DATABASE_PATH").unwrap_or_else(|_| "./data/lifeos.db".to_string()),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            token_expiry: env::var("JWT_EXPIRY")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600),
            refresh_token_expiry: env::var("REFRESH_EXPIRY")
                .unwrap_or_else(|_| "604800".to_string())
                .parse()
                .unwrap_or(604800),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            static_dir: env::var("STATIC_DIR").unwrap_or_else(|_| "../frontend/public".to_string()),
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        })
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

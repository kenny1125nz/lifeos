# LifeOS Backend

Rust-powered API server for LifeOS.

## Overview

The LifeOS backend is a Rust application responsible for serving static frontend files and providing user authentication services. It is designed for performance, security, and reliability.

---

## Target Platform

| Specification | Details |
|---------------|---------|
| **Architecture** | ARM64 (aarch64) |
| **Operating System** | Linux |
| **Target Device** | ARM64-based development boards (e.g., Raspberry Pi 4/5, Rock Pi, Orange Pi) |
| **Cross-Compilation** | Support for building on x86_64 host for ARM64 target |

### Cross-Compilation Setup

```bash
# Install ARM64 target
rustup target add aarch64-unknown-linux-gnu

# Install cross-compilation toolchain (Ubuntu/Debian)
sudo apt install gcc-aarch64-linux-gnu

# Build for ARM64
cargo build --release --target aarch64-unknown-linux-gnu
```

---

## Technology Stack

| Component | Technology |
|-----------|------------|
| **Language** | Rust (latest stable) |
| **Web Framework** | Axum or Actix-web |
| **Database** | SQLite (via rusqlite with bundled feature) |
| **Token Format** | JWT (jsonwebtoken crate) |
| **Password Hashing** | Argon2 (argon2 crate) |

---

## Persistence Layer Architecture

### Design Principles

The persistence layer is designed with **minimal coupling** to SQLite, enabling future migration to other databases or custom storage mechanisms.

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Services                      │
│         (auth.rs, token.rs, user_service.rs)                │
└─────────────────────────┬───────────────────────────────────┘
                          │ Uses trait interfaces
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                   Repository Traits                          │
│   UserRepository, TokenRepository, etc. (trait definitions) │
└─────────────────────────┬───────────────────────────────────┘
                          │ Implemented by
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  Repository Implementations                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   SQLite    │  │ PostgreSQL  │  │  Custom/In-Memory   │  │
│  │    Impl     │  │    Impl     │  │       Impl          │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Repository Pattern

All database operations are abstracted behind traits:

```rust
// Repository trait - database agnostic
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &NewUser) -> Result<User, RepoError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError>;
    async fn update(&self, user: &User) -> Result<User, RepoError>;
    async fn delete(&self, id: &str) -> Result<(), RepoError>;
}

#[async_trait]
pub trait TokenRepository: Send + Sync {
    async fn revoke(&self, jti: &str, expires_at: DateTime) -> Result<(), RepoError>;
    async fn is_revoked(&self, jti: &str) -> Result<bool, RepoError>;
    async fn cleanup_expired(&self) -> Result<u64, RepoError>;
}
```

### SQLite Implementation

The SQLite implementation uses `rusqlite` with bundled SQLite:

```toml
[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
```

**Benefits of bundled SQLite:**
- No external SQLite library dependency
- Consistent SQLite version across platforms
- Easier cross-compilation for ARM64
- Self-contained binary

### Automatic Database Initialization

The application automatically creates and initializes the database on startup:

```rust
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self, DbError> {
        // Create database file and parent directories if not exists
        if let Some(parent) = Path::new(db_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::builder().build(manager)?;
        
        // Run migrations on startup
        let conn = pool.get()?;
        Self::run_migrations(&conn)?;
        
        Ok(Self { pool })
    }
    
    fn run_migrations(conn: &Connection) -> Result<(), DbError> {
        conn.execute_batch(include_str!("../migrations/init.sql"))?;
        Ok(())
    }
}
```

### Migration Strategy

Embedded SQL migrations that run automatically:

```
backend/
├── migrations/
│   ├── init.sql           # Initial schema (always runs, idempotent)
│   └── migrations.sql     # Version-tracked migrations
```

### Swapping Persistence Implementations

To migrate to a different database or storage:

1. **Implement the repository traits** for the new backend
2. **Update dependency injection** in `main.rs`:

```rust
// Current: SQLite
let user_repo: Arc<dyn UserRepository> = Arc::new(SqliteUserRepository::new(&db));

// Future: PostgreSQL
let user_repo: Arc<dyn UserRepository> = Arc::new(PostgresUserRepository::new(&pool));

// Future: Custom file-based
let user_repo: Arc<dyn UserRepository> = Arc::new(FileUserRepository::new(&data_dir));
```

3. **No changes required** in services or routes

---

## Core Responsibilities

### 1. Static File Server

| Requirement | Description |
|-------------|-------------|
| **Asset Serving** | Serve HTML, CSS, JS, and media files |
| **Compression** | Support gzip/brotli compression |
| **Caching Headers** | Proper Cache-Control and ETag headers |
| **SPA Fallback** | Route non-file requests to `index.html` |

### 2. User Authentication

The backend provides complete user authentication and session management.

---

## Authentication System

### Sign-Up

- Email/password registration
- Input validation and sanitization
- Password hashing (Argon2)
- Email verification (optional, configurable)
- Duplicate email detection

### Sign-In

- Credential validation
- Rate limiting for brute-force protection
- Account lockout after failed attempts
- Secure password comparison (constant-time)

### Token Management

- **Token Issuing**: Generate JWT tokens on successful authentication
- **Token Validation**: Verify token signature and expiration
- **Token Refresh**: Support token renewal without re-authentication
- **Token Revocation**: Ability to invalidate tokens (logout, security events)

### Token Structure (JWT)

```json
{
  "sub": "user_id",
  "email": "user@example.com",
  "iat": 1234567890,
  "exp": 1234567890,
  "jti": "unique_token_id"
}
```

---

## API Endpoints

### Authentication

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/auth/signup` | Create new user account |
| `POST` | `/api/auth/signin` | Authenticate and receive token |
| `POST` | `/api/auth/refresh` | Refresh access token |
| `POST` | `/api/auth/logout` | Revoke current token |
| `GET` | `/api/auth/me` | Get current user info |

### Health & Status

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/health` | Service health check |
| `GET` | `/api/version` | API version information |

---

## Project Structure

```
backend/
├── src/
│   ├── main.rs              # Application entry point
│   ├── config.rs            # Configuration management
│   ├── error.rs             # Error types and handling
│   │
│   ├── routes/
│   │   ├── mod.rs           # Route aggregation
│   │   ├── auth.rs          # Authentication endpoints
│   │   └── static_files.rs  # Static file serving
│   │
│   ├── models/
│   │   ├── mod.rs
│   │   └── user.rs          # User model (domain objects)
│   │
│   ├── repositories/
│   │   ├── mod.rs           # Repository trait definitions
│   │   ├── traits.rs        # UserRepository, TokenRepository traits
│   │   └── sqlite/          # SQLite implementations
│   │       ├── mod.rs
│   │       ├── user_repo.rs
│   │       └── token_repo.rs
│   │
│   ├── services/
│   │   ├── mod.rs
│   │   ├── auth.rs          # Authentication logic
│   │   └── token.rs         # JWT token operations
│   │
│   └── middleware/
│       ├── mod.rs
│       └── auth.rs          # Authentication middleware
│
├── migrations/
│   └── init.sql             # Database initialization (idempotent)
│
├── Cargo.toml               # Rust dependencies
└── README.md                # This file
```

---

## Database Schema (SQLite)

### Users Table

```sql
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    verified INTEGER NOT NULL DEFAULT 0,
    locked INTEGER NOT NULL DEFAULT 0,
    failed_attempts INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
```

### Revoked Tokens Table

```sql
CREATE TABLE IF NOT EXISTS revoked_tokens (
    jti TEXT PRIMARY KEY,
    revoked_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_revoked_tokens_expires ON revoked_tokens(expires_at);
```

### Migrations Tracking Table

```sql
CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

---

## Configuration

Environment variables for configuration:

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_PATH` | Path to SQLite database file | `./data/lifeos.db` |
| `JWT_SECRET` | Secret key for JWT signing | (required) |
| `JWT_EXPIRY` | Token expiry duration (seconds) | `3600` (1 hour) |
| `REFRESH_EXPIRY` | Refresh token expiry (seconds) | `604800` (7 days) |
| `HOST` | Server bind address | `0.0.0.0` |
| `PORT` | Server port | `3000` |
| `STATIC_DIR` | Frontend static files directory | `../frontend/dist` |
| `RUST_LOG` | Log level | `info` |

---

## Development

### Prerequisites

- Rust (latest stable)
- No external SQLite installation required (bundled)

### Setup

```bash
cd backend

# Start development server (database auto-created)
cargo run
```

### Build for Production

```bash
# Native build
cargo build --release

# ARM64 cross-compile
cargo build --release --target aarch64-unknown-linux-gnu
```

### Running on ARM64 Device

```bash
# Copy binary to device
scp target/aarch64-unknown-linux-gnu/release/lifeos-backend user@device:/opt/lifeos/

# On the device
export JWT_SECRET="your-secret-key"
export DATABASE_PATH="/opt/lifeos/data/lifeos.db"
./lifeos-backend
```

---

## Security Considerations

- **Password Storage**: Argon2id with secure parameters
- **Token Security**: Short-lived access tokens, secure refresh tokens
- **Rate Limiting**: Prevent brute-force attacks
- **Input Validation**: Strict validation on all inputs
- **SQL Injection**: Parameterized queries via rusqlite
- **CORS**: Configurable cross-origin resource sharing
- **HTTPS**: Required in production (handled by reverse proxy)

---

## Error Response Format

All API errors follow a consistent format:

```json
{
  "error": {
    "code": "INVALID_CREDENTIALS",
    "message": "Invalid email or password"
  }
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_CREDENTIALS` | 401 | Wrong email or password |
| `TOKEN_EXPIRED` | 401 | JWT token has expired |
| `TOKEN_INVALID` | 401 | JWT token is malformed or invalid |
| `USER_EXISTS` | 409 | Email already registered |
| `VALIDATION_ERROR` | 400 | Input validation failed |
| `RATE_LIMITED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Unexpected server error |

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use tower_http::services::ServeDir;

pub fn create_router(static_dir: String) -> Router {
    let serve_dir = ServeDir::new(static_dir.clone())
        .append_index_html_on_directories(true)
        .fallback(ServeDir::new(static_dir).append_index_html_on_directories(true));

    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/version", get(version_info))
        .fallback_service(serve_dir)
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn version_info() -> impl IntoResponse {
    let version = env!("CARGO_PKG_VERSION");
    (StatusCode::OK, format!("LifeOS Backend v{}", version))
}

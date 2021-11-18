use axum::http::StatusCode;

/// Health check function. Returns 200
#[tracing::instrument(name = "Performing health check")]
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

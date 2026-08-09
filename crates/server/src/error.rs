use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// Wraps any error into a `500` JSON response `{"error": "..."}`.
///
/// Handlers return `Result<T, AppError>` and use `?` freely; the `From`
/// impl below accepts anything that converts into an `anyhow::Error`
/// (which covers `preflight_shared::Error`, axum's own multipart
/// errors, and everything else this crate produces).
pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // `{:#}` walks the full `.context(...)` chain (e.g. "replay
        // failed against 'new.so': failed to load program from
        // 'new.so': <the actual ELF loader error>") instead of just the
        // outermost message, which is what actually tells the user what
        // to fix about their upload.
        let message = format!("{:#}", self.0);
        tracing::error!(error = %message, "request failed");
        (StatusCode::BAD_REQUEST, Json(json!({ "error": message }))).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

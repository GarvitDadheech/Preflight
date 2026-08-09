mod error;
mod handlers;
mod state;

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use state::AppState;

const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "preflight_server=info,tower_http=info".into()),
        )
        .init();

    let state = AppState::new();

    let app = Router::new()
        .route("/api/health", get(handlers::health))
        .route("/api/run", post(handlers::run))
        .route("/api/demo", post(handlers::demo))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
        .layer(TraceLayer::new_for_http())
        // Permissive is fine here: this server is meant to run locally
        // for the dashboard's dev server to call, not to be exposed
        // publicly as-is.
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8787);
    let addr = format!("0.0.0.0:{port}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("preflight-server listening on http://{addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

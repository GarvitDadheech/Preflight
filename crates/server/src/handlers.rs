use anyhow::{anyhow, Context};
use axum::extract::{Multipart, State};
use axum::Json;
use preflight_shared::Report;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

/// `POST /api/run`: accepts a multipart form with two file fields,
/// `old` and `new`, replays the bundled transaction fixture against
/// both, and returns the resulting [`Report`] as JSON.
pub async fn run(mut multipart: Multipart) -> Result<Json<Report>, AppError> {
    let mut old: Option<(String, Vec<u8>)> = None;
    let mut new: Option<(String, Vec<u8>)> = None;

    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().unwrap_or("").to_string();
        let file_name = field
            .file_name()
            .map(str::to_string)
            .unwrap_or_else(|| format!("{field_name}.so"));

        match field_name.as_str() {
            "old" => old = Some((file_name, field.bytes().await?.to_vec())),
            "new" => new = Some((file_name, field.bytes().await?.to_vec())),
            _ => {
                // Ignore unrecognized fields rather than rejecting the
                // whole request over them.
                let _ = field.bytes().await;
            }
        }
    }

    let (old_name, old_bytes) = old.ok_or_else(|| anyhow!("missing 'old' file"))?;
    let (new_name, new_bytes) = new.ok_or_else(|| anyhow!("missing 'new' file"))?;

    let report = tokio::task::spawn_blocking(move || {
        replay_and_compare(&old_bytes, old_name, &new_bytes, new_name)
    })
    .await
    .context("replay task panicked")??;

    Ok(Json(report))
}

/// `POST /api/demo`: builds (or reuses a cached build of) the bundled
/// `examples/counter-old` and `examples/counter-new` programs and runs
/// the same replay + compare pipeline against them. Lets the dashboard
/// show a working example without requiring the user to have their own
/// `.so` files handy.
pub async fn demo(State(state): State<AppState>) -> Result<Json<Report>, AppError> {
    let programs = state.demo_programs().await?;

    let old_bytes = programs.old.clone();
    let new_bytes = programs.new.clone();

    let report = tokio::task::spawn_blocking(move || {
        replay_and_compare(
            &old_bytes,
            "counter-old (bundled example)".to_string(),
            &new_bytes,
            "counter-new (bundled example)".to_string(),
        )
    })
    .await
    .context("replay task panicked")??;

    Ok(Json(report))
}

fn replay_and_compare(
    old_bytes: &[u8],
    old_name: String,
    new_bytes: &[u8],
    new_name: String,
) -> anyhow::Result<Report> {
    let fixture = preflight_replay::generate_fixture();

    let old_results = preflight_replay::replay_bytes(old_bytes, old_name.clone(), &fixture, |_, _| {})
        .with_context(|| format!("replay failed against '{old_name}'"))?;
    let new_results = preflight_replay::replay_bytes(new_bytes, new_name.clone(), &fixture, |_, _| {})
        .with_context(|| format!("replay failed against '{new_name}'"))?;

    let report = preflight_comparator::compare(
        &old_name,
        &new_name,
        &fixture.transactions,
        &old_results,
        &new_results,
    )
    .context("failed to compare replay results")?;

    Ok(report)
}

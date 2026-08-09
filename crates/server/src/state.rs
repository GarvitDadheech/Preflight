use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::OnceCell;

/// platform-tools version passed to `cargo build-sbf` when building the
/// bundled demo programs. See the root README for why this is pinned.
const TOOLS_VERSION: &str = "v1.54";

/// The two bundled example programs, built once on first use and then
/// reused for every `/api/demo` request.
pub struct DemoPrograms {
    pub old: Vec<u8>,
    pub new: Vec<u8>,
}

#[derive(Clone)]
pub struct AppState {
    demo: Arc<OnceCell<DemoPrograms>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            demo: Arc::new(OnceCell::new()),
        }
    }

    /// Returns the bundled example programs, building them with `cargo
    /// build-sbf` the first time this is called.
    pub async fn demo_programs(&self) -> Result<&DemoPrograms> {
        self.demo
            .get_or_try_init(|| async {
                tracing::info!("building bundled example programs (first /api/demo request)");
                tokio::task::spawn_blocking(build_demo_programs).await?
            })
            .await
    }
}

fn build_demo_programs() -> Result<DemoPrograms> {
    let out_dir: PathBuf = std::env::temp_dir().join("preflight-server-demo-programs");

    let old_path =
        preflight_cli::build::build_example_program("counter-old", &out_dir, TOOLS_VERSION)
            .context("failed to build examples/counter-old")?;
    let new_path =
        preflight_cli::build::build_example_program("counter-new", &out_dir, TOOLS_VERSION)
            .context("failed to build examples/counter-new")?;

    Ok(DemoPrograms {
        old: std::fs::read(&old_path)
            .with_context(|| format!("failed to read {}", old_path.display()))?,
        new: std::fs::read(&new_path)
            .with_context(|| format!("failed to read {}", new_path.display()))?,
    })
}

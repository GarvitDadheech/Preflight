//! Library half of the `preflight` CLI.
//!
//! Split out so `preflight-server` can reuse the same "build the bundled
//! example programs" and "replay + compare" orchestration the binary
//! uses, instead of re-implementing it behind an HTTP API.

pub mod build;
pub mod pipeline;

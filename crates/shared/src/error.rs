use thiserror::Error;

/// Error type shared by every Preflight crate.
///
/// Kept in one place because `shared` already sits underneath `replay`,
/// `comparator`, `report` and `cli` for its data types; giving each of
/// those crates its own near-identical error enum would just be
/// boilerplate without adding real isolation.
#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to (de)serialize JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("failed to build example program '{name}': {reason}")]
    ProgramBuild { name: String, reason: String },

    #[error("failed to load program from '{path}': {reason}")]
    ProgramLoad { path: String, reason: String },

    #[error("transaction '{label}' could not be built: {reason}")]
    TransactionBuild { label: String, reason: String },

    #[error("invalid keypair bytes for '{label}'")]
    InvalidKeypair { label: String },

    #[error("invalid pubkey '{0}'")]
    InvalidPubkey(String),

    #[error("mismatched replay results: {0}")]
    Mismatch(String),
}

pub type Result<T> = std::result::Result<T, Error>;

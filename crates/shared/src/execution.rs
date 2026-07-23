use serde::{Deserialize, Serialize};

/// Decoded on-chain state of the counter account after a transaction.
///
/// `None` on a `TxExecutionResult` means the transaction failed before
/// any state existed to read (or the account could not be deserialized),
/// not that the value is zero.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CounterStateSnapshot {
    pub is_initialized: bool,
    pub authority: String,
    pub value: u64,
}

/// The observable outcome of replaying one transaction against one
/// program build.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TxExecutionResult {
    pub label: String,
    pub success: bool,
    /// Human-readable error description, e.g. "Custom program error 0x1"
    /// or a decoded `TransactionError` variant. `None` on success.
    pub error: Option<String>,
    pub logs: Vec<String>,
    pub compute_units_consumed: u64,
    pub counter_state: Option<CounterStateSnapshot>,
}

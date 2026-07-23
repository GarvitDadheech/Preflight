//! Mirrors the on-chain layout of `examples/counter-old` and
//! `examples/counter-new`.
//!
//! The two program crates are built for the SBF target by a different
//! toolchain (`cargo build-sbf`) and are intentionally isolated from this
//! workspace, so `replay` cannot simply `use` their types. Instead it
//! keeps a byte-for-byte compatible copy of the instruction and state
//! layout here. Both example programs share this layout; only their
//! validation logic differs.
//!
//! A production Preflight would replace this with something driven by
//! the target program's IDL (for Anchor programs) or a user-supplied
//! schema, rather than a hardcoded copy.

use borsh::{BorshDeserialize, BorshSerialize};
use preflight_shared::CounterInstructionSpec;
use solana_pubkey::Pubkey;

/// Must match `examples/*/src/instruction.rs::CounterInstruction`
/// exactly: same variants, same field names, same order.
#[derive(BorshSerialize)]
pub enum CounterInstruction {
    Initialize { start_value: u64 },
    Increment { amount: u64 },
    Decrement { amount: u64 },
    SetValue { value: u64 },
}

impl From<&CounterInstructionSpec> for CounterInstruction {
    fn from(spec: &CounterInstructionSpec) -> Self {
        match *spec {
            CounterInstructionSpec::Initialize { start_value } => {
                Self::Initialize { start_value }
            }
            CounterInstructionSpec::Increment { amount } => Self::Increment { amount },
            CounterInstructionSpec::Decrement { amount } => Self::Decrement { amount },
            CounterInstructionSpec::SetValue { value } => Self::SetValue { value },
        }
    }
}

/// Must match `examples/*/src/state.rs::Counter` exactly.
#[derive(BorshDeserialize, Default)]
pub struct Counter {
    pub is_initialized: bool,
    pub authority: Pubkey,
    pub value: u64,
}

impl Counter {
    /// Must match `examples/*/src/state.rs::Counter::LEN`.
    pub const LEN: usize = 1 + 32 + 8;
}

/// Human-readable names for the custom program error codes defined in
/// `examples/counter-new/src/error.rs`. The baseline program never
/// returns a custom error, so this table only ever applies to the new
/// build.
pub fn describe_custom_error(code: u32) -> Option<&'static str> {
    match code {
        0 => Some("InvalidAuthority"),
        1 => Some("ValueExceedsMax"),
        2 => Some("Underflow"),
        _ => None,
    }
}

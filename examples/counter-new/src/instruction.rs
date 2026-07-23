use borsh::{BorshDeserialize, BorshSerialize};

/// Instructions understood by the counter program.
///
/// Account layout is identical across every instruction so that a single
/// `AccountMeta` list shape can be reused by callers:
///   0. `[writable]` The counter account.
///   1. `[signer]`   The authority.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum CounterInstruction {
    /// Create and initialize a counter account with a starting value.
    Initialize { start_value: u64 },
    /// Add `amount` to the current value.
    Increment { amount: u64 },
    /// Subtract `amount` from the current value.
    Decrement { amount: u64 },
    /// Overwrite the current value directly. Restricted to the authority.
    SetValue { value: u64 },
}

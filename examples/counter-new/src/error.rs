use solana_program::program_error::ProgramError;

/// Errors introduced by the new validation added in this version.
///
/// Failures that already existed in the baseline program (uninitialized
/// account, missing signature) keep returning the same stock
/// `ProgramError` variants; only the newly added checks get dedicated
/// codes here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CounterError {
    /// The signer does not match the counter's stored authority.
    InvalidAuthority = 0,
    /// The requested value would exceed `MAX_VALUE`.
    ValueExceedsMax = 1,
    /// The requested decrement is larger than the current value.
    Underflow = 2,
}

impl From<CounterError> for ProgramError {
    fn from(error: CounterError) -> Self {
        ProgramError::Custom(error as u32)
    }
}

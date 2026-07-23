use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// On-chain state for a single counter account.
#[derive(BorshSerialize, BorshDeserialize, Debug, Default, PartialEq, Eq)]
pub struct Counter {
    pub is_initialized: bool,
    pub authority: Pubkey,
    pub value: u64,
}

impl Counter {
    /// Fixed serialized size in bytes: 1 (bool) + 32 (pubkey) + 8 (u64).
    pub const LEN: usize = 1 + 32 + 8;
}

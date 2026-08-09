//! Executes a [`preflight_shared::Fixture`]'s transaction sequence
//! against a compiled program, inside an in-process local Solana VM
//! ([litesvm](https://github.com/LiteSVM/litesvm)).
//!
//! Everything here runs locally and in-memory: no RPC calls, no
//! external validator process, no network access.

mod engine;
mod keys;
mod program_abi;

pub use engine::{replay, replay_bytes};

use preflight_shared::{Fixture, KeypairBytes};
use solana_keypair::Keypair;

/// Builds a fresh fixture: a new set of keypairs plus the fixed example
/// transaction sequence from [`Fixture::example_transactions`].
///
/// Generating fresh keypairs on every `preflight run` (rather than
/// reusing a checked-in set) keeps each run's local VM state
/// self-contained; what matters for a valid comparison is that the same
/// fixture is replayed against both program builds, not that it is
/// stable across separate invocations of the tool.
pub fn generate_fixture() -> Fixture {
    Fixture {
        payer: to_keypair_bytes(Keypair::new()),
        authority: to_keypair_bytes(Keypair::new()),
        rogue: to_keypair_bytes(Keypair::new()),
        counter_account: to_keypair_bytes(Keypair::new()),
        transactions: Fixture::example_transactions(),
    }
}

fn to_keypair_bytes(keypair: Keypair) -> KeypairBytes {
    KeypairBytes(keypair.to_bytes().to_vec())
}

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Raw bytes of an ed25519 keypair, in the same 64-byte layout
/// `solana-keygen` writes to a keypair JSON file. Kept as raw bytes here
/// (rather than a `solana_keypair::Keypair`) so this crate has no
/// dependency on the Solana runtime — only `replay` needs to turn these
/// back into signers.
///
/// Stored as a `Vec` rather than `[u8; 64]` purely because `serde`'s
/// built-in array support tops out at 32 elements; callers still expect
/// exactly 64 bytes.
#[derive(Serialize, Deserialize, Clone)]
pub struct KeypairBytes(pub Vec<u8>);

/// Which role signs the "authority" account slot of an instruction.
///
/// The counter program always expects `[counter_account, authority]` as
/// its accounts. Most instructions are signed by the real authority;
/// `Rogue` deliberately signs with a different keypair so the fixture can
/// exercise the "wrong signer" rejection path in both program versions.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignerRole {
    Authority,
    Rogue,
}

/// A description of one of the counter program's instructions.
///
/// This mirrors the on-chain `CounterInstruction` enum byte-for-byte
/// (same variants, same field order) so `replay` can Borsh-encode it into
/// instruction data without linking against the program crate itself,
/// which is built for a different target by a different toolchain.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CounterInstructionSpec {
    Initialize { start_value: u64 },
    Increment { amount: u64 },
    Decrement { amount: u64 },
    SetValue { value: u64 },
}

/// One transaction in the replay sequence.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionSpec {
    /// Short, stable, human-readable identifier used in reports.
    pub label: String,
    /// What this transaction is expected to demonstrate, shown in the
    /// Markdown report next to its result.
    pub description: String,
    pub instruction: CounterInstructionSpec,
    pub signer: SignerRole,
}

/// Everything needed to replay the same sequence of transactions against
/// an arbitrary build of the counter program.
///
/// Persisted to `transactions.json` after the first (recording) run so
/// the exact same fixture, including keypairs, can be reused for the
/// second run without regenerating anything.
#[derive(Serialize, Deserialize, Clone)]
pub struct Fixture {
    pub payer: KeypairBytes,
    pub authority: KeypairBytes,
    pub rogue: KeypairBytes,
    pub counter_account: KeypairBytes,
    pub transactions: Vec<TransactionSpec>,
}

impl Fixture {
    /// Persists the fixture (including keypairs) to disk so the exact
    /// same transactions can be replayed again later without
    /// regenerating anything.
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Loads a fixture previously written by [`Fixture::save`].
    pub fn load(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }

    /// The example transaction sequence used by `preflight demo`.
    ///
    /// Chosen to exercise every intentional behavior change between
    /// `examples/counter-old` and `examples/counter-new`: a value cap on
    /// `Increment`/`SetValue`, a hard underflow error on `Decrement`, and
    /// a different error code for a wrong-authority `SetValue`.
    pub fn example_transactions() -> Vec<TransactionSpec> {
        vec![
            TransactionSpec {
                label: "initialize".into(),
                description: "Create the counter starting at 10.".into(),
                instruction: CounterInstructionSpec::Initialize { start_value: 10 },
                signer: SignerRole::Authority,
            },
            TransactionSpec {
                label: "increment_50".into(),
                description: "Increment by 50 (10 -> 60). Within limits in both versions.".into(),
                instruction: CounterInstructionSpec::Increment { amount: 50 },
                signer: SignerRole::Authority,
            },
            TransactionSpec {
                label: "increment_20".into(),
                description: "Increment by 20 (60 -> 80). Still within limits.".into(),
                instruction: CounterInstructionSpec::Increment { amount: 20 },
                signer: SignerRole::Authority,
            },
            TransactionSpec {
                label: "decrement_30".into(),
                description: "Decrement by 30 (80 -> 50). No underflow, unaffected by the new version's stricter check.".into(),
                instruction: CounterInstructionSpec::Decrement { amount: 30 },
                signer: SignerRole::Authority,
            },
            TransactionSpec {
                label: "decrement_past_zero".into(),
                description: "Decrement by 1000 from 50. The old program clamps to 0; the new program treats this as an underflow error.".into(),
                instruction: CounterInstructionSpec::Decrement { amount: 1000 },
                signer: SignerRole::Authority,
            },
            TransactionSpec {
                label: "increment_past_cap".into(),
                description: "Increment by 2000. The old program has no upper bound; the new program rejects values over 1000.".into(),
                instruction: CounterInstructionSpec::Increment { amount: 2000 },
                signer: SignerRole::Authority,
            },
            TransactionSpec {
                label: "set_value_over_cap".into(),
                description: "Set the value to 5000. Allowed in the old program, rejected by the new program's value cap.".into(),
                instruction: CounterInstructionSpec::SetValue { value: 5000 },
                signer: SignerRole::Authority,
            },
            TransactionSpec {
                label: "set_value_wrong_authority".into(),
                description: "Attempt to set the value while signing with a non-authority key. Fails in both versions, but with a different error code.".into(),
                instruction: CounterInstructionSpec::SetValue { value: 1 },
                signer: SignerRole::Rogue,
            },
            TransactionSpec {
                label: "increment_10_final".into(),
                description: "Increment by 10. Included to show how the two versions' account state has diverged by the end of the sequence.".into(),
                instruction: CounterInstructionSpec::Increment { amount: 10 },
                signer: SignerRole::Authority,
            },
        ]
    }
}

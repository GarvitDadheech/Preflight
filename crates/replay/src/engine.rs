use std::path::Path;

use litesvm::LiteSVM;
use preflight_shared::{
    CounterStateSnapshot, Error, Fixture, Result, SignerRole, TransactionSpec, TxExecutionResult,
};
use solana_instruction::{account_meta::AccountMeta, Instruction};
use solana_instruction_error::InstructionError;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_system_interface::instruction as system_instruction;
use solana_transaction::Transaction;
use solana_transaction_error::TransactionError;

use crate::keys::keypair_from_bytes;
use crate::program_abi::{describe_custom_error, Counter, CounterInstruction};

/// A fixed, deterministic program id used to load whichever `.so` is
/// under test into the local VM.
///
/// It is not a real deployment address: every replay run gets its own
/// fresh, isolated litesvm instance, so there is nothing for it to
/// collide with.
fn program_id() -> Pubkey {
    Pubkey::new_from_array([7u8; 32])
}

/// Executes `fixture`'s transaction sequence against the program at
/// `program_path` inside a fresh, in-process litesvm instance.
///
/// `on_step` is invoked after each transaction executes, so callers can
/// report progress as the replay proceeds.
pub fn replay(
    program_path: &Path,
    fixture: &Fixture,
    mut on_step: impl FnMut(&TransactionSpec, &TxExecutionResult),
) -> Result<Vec<TxExecutionResult>> {
    let program_id = program_id();

    let mut svm = LiteSVM::new();
    svm.add_program_from_file(program_id, program_path)
        .map_err(|e| Error::ProgramLoad {
            path: program_path.display().to_string(),
            reason: e.to_string(),
        })?;

    let payer = keypair_from_bytes("payer", &fixture.payer)?;
    let authority = keypair_from_bytes("authority", &fixture.authority)?;
    let rogue = keypair_from_bytes("rogue", &fixture.rogue)?;
    let counter_kp = keypair_from_bytes("counter_account", &fixture.counter_account)?;

    fund(&mut svm, &payer.pubkey())?;
    fund(&mut svm, &authority.pubkey())?;
    fund(&mut svm, &rogue.pubkey())?;
    create_counter_account(&mut svm, &payer, &counter_kp, &program_id)?;

    let blockhash = svm.latest_blockhash();
    let mut results = Vec::with_capacity(fixture.transactions.len());

    for spec in &fixture.transactions {
        let role_kp: &Keypair = match spec.signer {
            SignerRole::Authority => &authority,
            SignerRole::Rogue => &rogue,
        };

        let data = borsh::to_vec(&CounterInstruction::from(&spec.instruction)).map_err(|e| {
            Error::TransactionBuild {
                label: spec.label.clone(),
                reason: e.to_string(),
            }
        })?;

        let instruction = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(counter_kp.pubkey(), false),
                AccountMeta::new_readonly(role_kp.pubkey(), true),
            ],
            data,
        };

        let message = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
        let tx = Transaction::new(&[&payer, role_kp], message, blockhash);

        let outcome = svm.send_transaction(tx);
        let result = to_execution_result(spec, outcome, &svm, &counter_kp.pubkey());
        on_step(spec, &result);
        results.push(result);
    }

    Ok(results)
}

fn fund(svm: &mut LiteSVM, pubkey: &Pubkey) -> Result<()> {
    svm.airdrop(pubkey, 10 * LAMPORTS_PER_SOL)
        .map_err(|e| Error::TransactionBuild {
            label: "airdrop".to_string(),
            reason: format!("{:?}", e),
        })?;
    Ok(())
}

fn create_counter_account(
    svm: &mut LiteSVM,
    payer: &Keypair,
    counter_kp: &Keypair,
    program_id: &Pubkey,
) -> Result<()> {
    let rent = svm.minimum_balance_for_rent_exemption(Counter::LEN);
    let ix = system_instruction::create_account(
        &payer.pubkey(),
        &counter_kp.pubkey(),
        rent,
        Counter::LEN as u64,
        program_id,
    );

    let blockhash = svm.latest_blockhash();
    let message = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = Transaction::new(&[payer, counter_kp], message, blockhash);

    svm.send_transaction(tx).map_err(|e| Error::TransactionBuild {
        label: "create_counter_account".to_string(),
        reason: format!("{:?}", e),
    })?;
    Ok(())
}

fn to_execution_result(
    spec: &TransactionSpec,
    outcome: litesvm::types::TransactionResult,
    svm: &LiteSVM,
    counter_pubkey: &Pubkey,
) -> TxExecutionResult {
    let counter_state = decode_counter_state(svm, counter_pubkey);

    match outcome {
        Ok(meta) => TxExecutionResult {
            label: spec.label.clone(),
            success: true,
            error: None,
            logs: meta.logs,
            compute_units_consumed: meta.compute_units_consumed,
            counter_state,
        },
        Err(failed) => TxExecutionResult {
            label: spec.label.clone(),
            success: false,
            error: Some(describe_transaction_error(&failed.err)),
            logs: failed.meta.logs,
            compute_units_consumed: failed.meta.compute_units_consumed,
            counter_state,
        },
    }
}

fn decode_counter_state(svm: &LiteSVM, counter_pubkey: &Pubkey) -> Option<CounterStateSnapshot> {
    let account = svm.get_account(counter_pubkey)?;
    let counter: Counter = borsh::from_slice(&account.data).ok()?;
    Some(CounterStateSnapshot {
        is_initialized: counter.is_initialized,
        authority: counter.authority.to_string(),
        value: counter.value,
    })
}

/// Formats a `TransactionError` for the report, decoding custom program
/// error codes from the bundled example program where possible.
fn describe_transaction_error(err: &TransactionError) -> String {
    if let TransactionError::InstructionError(index, InstructionError::Custom(code)) = err {
        return match describe_custom_error(*code) {
            Some(name) => format!("instruction {index} failed: custom program error {code} ({name})"),
            None => format!("instruction {index} failed: custom program error {code}"),
        };
    }
    format!("{err:?}")
}

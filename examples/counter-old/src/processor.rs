use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{instruction::CounterInstruction, state::Counter};

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = CounterInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let account_info_iter = &mut accounts.iter();
    let counter_account = next_account_info(account_info_iter)?;
    let authority = next_account_info(account_info_iter)?;

    match instruction {
        CounterInstruction::Initialize { start_value } => {
            initialize(counter_account, authority, start_value)
        }
        CounterInstruction::Increment { amount } => increment(counter_account, amount),
        CounterInstruction::Decrement { amount } => decrement(counter_account, amount),
        CounterInstruction::SetValue { value } => set_value(counter_account, authority, value),
    }
}

fn initialize(
    counter_account: &AccountInfo,
    authority: &AccountInfo,
    start_value: u64,
) -> ProgramResult {
    let mut counter = Counter::try_from_slice(&counter_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if counter.is_initialized {
        msg!("counter: already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    counter.is_initialized = true;
    counter.authority = *authority.key;
    counter.value = start_value;

    counter.serialize(&mut &mut counter_account.data.borrow_mut()[..])?;
    msg!("counter: initialized with value {}", start_value);
    Ok(())
}

fn increment(counter_account: &AccountInfo, amount: u64) -> ProgramResult {
    let mut counter = load_initialized(counter_account)?;

    counter.value = counter
        .value
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    counter.serialize(&mut &mut counter_account.data.borrow_mut()[..])?;
    msg!("counter: incremented by {} to {}", amount, counter.value);
    Ok(())
}

fn decrement(counter_account: &AccountInfo, amount: u64) -> ProgramResult {
    let mut counter = load_initialized(counter_account)?;

    // Baseline behavior: decrementing past zero saturates at zero rather
    // than failing.
    counter.value = counter.value.saturating_sub(amount);

    counter.serialize(&mut &mut counter_account.data.borrow_mut()[..])?;
    msg!("counter: decremented by {} to {}", amount, counter.value);
    Ok(())
}

fn set_value(counter_account: &AccountInfo, authority: &AccountInfo, value: u64) -> ProgramResult {
    let mut counter = load_initialized(counter_account)?;

    if !authority.is_signer {
        msg!("counter: authority did not sign");
        return Err(ProgramError::MissingRequiredSignature);
    }
    if counter.authority != *authority.key {
        msg!("counter: signer is not the counter authority");
        return Err(ProgramError::InvalidArgument);
    }

    counter.value = value;

    counter.serialize(&mut &mut counter_account.data.borrow_mut()[..])?;
    msg!("counter: value set to {}", value);
    Ok(())
}

fn load_initialized(counter_account: &AccountInfo) -> Result<Counter, ProgramError> {
    let counter = Counter::try_from_slice(&counter_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;
    if !counter.is_initialized {
        msg!("counter: account is not initialized");
        return Err(ProgramError::UninitializedAccount);
    }
    Ok(counter)
}

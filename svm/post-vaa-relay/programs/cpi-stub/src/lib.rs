#![no_std]
//! Test double for the core bridge `post_vaa` CPI target.
//!
//! Return data layout: for each account, 32-byte key then one flag byte
//! ([`FLAG_WRITABLE`] | [`FLAG_SIGNER`]); then the instruction data verbatim.
//! Fails with `InvalidInstructionData` when the echo exceeds
//! [`MAX_RETURN_DATA`].

use pinocchio::{
    account_info::AccountInfo, cpi::set_return_data, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};

pinocchio::program_entrypoint!(process_instruction);
pinocchio::default_allocator!();
pinocchio::nostd_panic_handler!();

/// Runtime limit for return data.
pub const MAX_RETURN_DATA: usize = 1024;
/// Bytes per echoed account: key plus flag byte.
pub const ACCOUNT_RECORD_LEN: usize = 33;
pub const FLAG_WRITABLE: u8 = 1;
pub const FLAG_SIGNER: u8 = 2;

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let total = accounts
        .len()
        .checked_mul(ACCOUNT_RECORD_LEN)
        .and_then(|n| n.checked_add(instruction_data.len()))
        .ok_or(ProgramError::InvalidInstructionData)?;
    if total > MAX_RETURN_DATA {
        return Err(ProgramError::InvalidInstructionData);
    }

    let mut buf = [0u8; MAX_RETURN_DATA];
    let mut offset = 0;
    for account in accounts {
        buf[offset..offset + 32].copy_from_slice(account.key());
        let mut flags = 0;
        if account.is_writable() {
            flags |= FLAG_WRITABLE;
        }
        if account.is_signer() {
            flags |= FLAG_SIGNER;
        }
        buf[offset + 32] = flags;
        offset += ACCOUNT_RECORD_LEN;
    }
    buf[offset..total].copy_from_slice(instruction_data);

    set_return_data(&buf[..total]);
    Ok(())
}

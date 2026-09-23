#![no_std]
//! Throwaway devnet proof. CPIs into the real Wormhole core bridge's
//! `post_vaa`, downgrading `signature_set` from writable to read-only for
//! that call, so a transaction containing both `verify_signatures` (needs
//! `signature_set` writable) and this relay (asks for it read-only) fits in
//! one transaction.
//!
//! One instruction. The instruction data passed in is forwarded verbatim as
//! `post_vaa`'s own data (a 1-byte discriminator + Borsh-encoded
//! `PostVAAData`).
//!
//! Accounts, in order (matching @wormhole-foundation/sdk-solana-core's
//! `getPostVaaAccounts`, which is the proven-correct account list -- the
//! `PostVAA` Rust struct's named fields alone omit `rent`/`system_program`,
//! needed for `message`'s first-time account creation):
//! 0. core_bridge_program (readonly, executable)
//! 1. guardian_set (readonly)
//! 2. bridge_info (readonly)
//! 3. signature_set (downgraded to readonly for the CPI)
//! 4. message (writable -- the PostedVAA PDA `post_vaa` creates)
//! 5. payer (writable, signer)
//! 6. clock sysvar (readonly)
//! 7. rent sysvar (readonly)
//! 8. system_program (readonly, executable)

use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke,
    default_allocator,
    instruction::{AccountMeta, Instruction},
    nostd_panic_handler, program_entrypoint,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

program_entrypoint!(process_instruction);
default_allocator!();
nostd_panic_handler!();

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [core_bridge_program, guardian_set, bridge_info, signature_set, message, payer, clock, rent, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let cpi_instruction = Instruction {
        program_id: core_bridge_program.key(),
        accounts: &[
            AccountMeta {
                pubkey: guardian_set.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: bridge_info.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: signature_set.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: message.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: payer.key(),
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: clock.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: rent.key(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: system_program.key(),
                is_signer: false,
                is_writable: false,
            },
        ],
        data: instruction_data,
    };

    invoke(
        &cpi_instruction,
        &[
            guardian_set,
            bridge_info,
            signature_set,
            message,
            payer,
            clock,
            rent,
            system_program,
        ],
    )
}

#![no_std]
//! Relay for the Wormhole core bridge `post_vaa` instruction.
//!
//! Forwards the caller's accounts and data through one CPI, with
//! `signature_set` downgraded to read-only. This lets `verify_signatures`
//! (needs `signature_set` writable) and `post_vaa` share one transaction.
//!
//! Instruction data is forwarded verbatim: [`POST_VAA_DISCRIMINATOR`]
//! followed by Borsh `PostVAAData`.
//!
//! Accounts, in `@wormhole-foundation/sdk-solana-core` `getPostVaaAccounts`
//! order:
//! 0. core_bridge_program (readonly, executable, key == [`CORE_BRIDGE_ID`])
//! 1. guardian_set (readonly)
//! 2. bridge_info (readonly)
//! 3. signature_set (readonly for the CPI)
//! 4. message (writable, PostedVAA PDA created by `post_vaa`)
//! 5. payer (writable, signer)
//! 6. clock sysvar (readonly)
//! 7. rent sysvar (readonly)
//! 8. system_program (readonly)
//!
//! `post_vaa` creates `message` through System Program CPIs resolved against
//! its own account list, so `system_program` must be present. `rent` keeps
//! wire parity with the SDK builder.

use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

pinocchio::program_entrypoint!(process_instruction);
pinocchio::default_allocator!();
pinocchio::nostd_panic_handler!();

/// Solitaire `Instruction::PostVAA` enum index in the core bridge.
pub const POST_VAA_DISCRIMINATOR: u8 = 2;

/// Accounts the relay takes: the core bridge program plus the eight
/// `post_vaa` accounts.
pub const ACCOUNT_COUNT: usize = 9;

/// Accounts forwarded to `post_vaa`.
pub const CPI_ACCOUNT_COUNT: usize = 8;

/// Core bridge program id the relay accepts as CPI target, from the
/// compile-time environment. Set `CORE_BRIDGE_ADDRESS` to a base58 program
/// id; `just build <net>` sets it for the known networks.
pub const CORE_BRIDGE_ID_BASE58: &str = env!(
    "CORE_BRIDGE_ADDRESS",
    "set CORE_BRIDGE_ADDRESS to the core bridge program id (base58); `just build mainnet|testnet|localnet` does this"
);
pub const CORE_BRIDGE_ID: Pubkey = decode_base58_pubkey(CORE_BRIDGE_ID_BASE58);

/// Decodes a base58 string into exactly 32 bytes. Evaluated at compile time
/// for [`CORE_BRIDGE_ID`]; panics on an invalid character, a value above 32
/// bytes, or a non-canonical encoding (leading `1`s must equal leading zero bytes).
pub const fn decode_base58_pubkey(s: &str) -> Pubkey {
    const ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let bytes = s.as_bytes();
    assert!(bytes.len() >= 32, "base58 pubkey too short");
    assert!(bytes.len() <= 44, "base58 pubkey too long");

    let mut out = [0u8; 32];
    let mut i = 0;
    while i < bytes.len() {
        let mut digit = 58;
        let mut j = 0;
        while j < ALPHABET.len() {
            if ALPHABET[j] == bytes[i] {
                digit = j;
                break;
            }
            j += 1;
        }
        assert!(digit < 58, "invalid base58 character");

        // out = out * 58 + digit, big-endian.
        let mut carry = digit;
        let mut k = out.len();
        while k > 0 {
            k -= 1;
            let v = out[k] as usize * 58 + carry;
            out[k] = (v & 0xff) as u8;
            carry = v >> 8;
        }
        assert!(carry == 0, "base58 value exceeds 32 bytes");
        i += 1;
    }

    let mut leading_ones = 0;
    while leading_ones < bytes.len() && bytes[leading_ones] == b'1' {
        leading_ones += 1;
    }
    let mut leading_zeros = 0;
    while leading_zeros < out.len() && out[leading_zeros] == 0 {
        leading_zeros += 1;
    }
    assert!(
        leading_ones == leading_zeros,
        "non-canonical base58 for 32 bytes"
    );
    out
}

/// Preconditions: exactly [`ACCOUNT_COUNT`] accounts, account 0 is the
/// executable [`CORE_BRIDGE_ID`], data starts with [`POST_VAA_DISCRIMINATOR`].
/// Writable and signer flags of accounts 4 and 5 are enforced by the runtime
/// at CPI time.
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [core_bridge_program, guardian_set, bridge_info, signature_set, message, payer, clock, rent, system_program] =
        accounts
    else {
        return Err(if accounts.len() < ACCOUNT_COUNT {
            ProgramError::NotEnoughAccountKeys
        } else {
            ProgramError::InvalidArgument
        });
    };

    if core_bridge_program.key() != &CORE_BRIDGE_ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    if !core_bridge_program.executable() {
        return Err(ProgramError::InvalidAccountData);
    }
    if instruction_data.first() != Some(&POST_VAA_DISCRIMINATOR) {
        return Err(ProgramError::InvalidInstructionData);
    }

    let cpi_accounts: [AccountMeta; CPI_ACCOUNT_COUNT] = [
        AccountMeta::readonly(guardian_set.key()),
        AccountMeta::readonly(bridge_info.key()),
        // Downgraded: the core bridge declares it read-only.
        AccountMeta::readonly(signature_set.key()),
        AccountMeta::writable(message.key()),
        AccountMeta::writable_signer(payer.key()),
        AccountMeta::readonly(clock.key()),
        AccountMeta::readonly(rent.key()),
        AccountMeta::readonly(system_program.key()),
    ];

    let cpi_instruction = Instruction {
        program_id: &CORE_BRIDGE_ID,
        accounts: &cpi_accounts,
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

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

/// Core bridge program ids per network. Byte arrays are the base58 decoded
/// keys; `tests/relay.rs` asserts each pair matches.
pub mod core_bridge_ids {
    use pinocchio::pubkey::Pubkey;

    pub const MAINNET: Pubkey = [
        14, 10, 88, 154, 65, 165, 95, 189, 102, 197, 42, 71, 95, 45, 146, 166, 211, 220, 155, 71,
        71, 17, 76, 185, 175, 130, 90, 152, 181, 69, 211, 206,
    ];
    pub const MAINNET_BASE58: &str = "worm2ZoG2kUd4vFXhvjh93UUH596ayRfgQ2MgjNMTth";

    /// Wormhole "testnet" is the Solana devnet cluster.
    pub const TESTNET: Pubkey = [
        43, 18, 70, 201, 238, 250, 60, 70, 103, 146, 37, 49, 17, 243, 95, 236, 30, 232, 238, 94,
        157, 235, 196, 18, 210, 233, 173, 173, 254, 205, 204, 114,
    ];
    pub const TESTNET_BASE58: &str = "3u8hJUVTA4jH1wYAyUur7FFZVQ8H635K3tSHHF4ssjQ5";

    /// Tilt devnet.
    pub const LOCALNET: Pubkey = [
        2, 200, 6, 49, 44, 190, 91, 121, 239, 138, 166, 193, 126, 63, 66, 61, 143, 223, 225, 212,
        105, 9, 251, 31, 108, 223, 101, 238, 142, 46, 111, 170,
    ];
    pub const LOCALNET_BASE58: &str = "Bridge1p5gheXUvJ6jGWGeCsgPKgnE3YgdGKRVCMY9o";
}

/// Core bridge program id for the selected network feature.
#[cfg(feature = "mainnet")]
pub const CORE_BRIDGE_ID: Pubkey = core_bridge_ids::MAINNET;
#[cfg(feature = "mainnet")]
pub const CORE_BRIDGE_ID_BASE58: &str = core_bridge_ids::MAINNET_BASE58;

/// Core bridge program id for the selected network feature.
#[cfg(feature = "testnet")]
pub const CORE_BRIDGE_ID: Pubkey = core_bridge_ids::TESTNET;
#[cfg(feature = "testnet")]
pub const CORE_BRIDGE_ID_BASE58: &str = core_bridge_ids::TESTNET_BASE58;

/// Core bridge program id for the selected network feature.
#[cfg(feature = "localnet")]
pub const CORE_BRIDGE_ID: Pubkey = core_bridge_ids::LOCALNET;
#[cfg(feature = "localnet")]
pub const CORE_BRIDGE_ID_BASE58: &str = core_bridge_ids::LOCALNET_BASE58;

#[cfg(not(any(feature = "mainnet", feature = "testnet", feature = "localnet")))]
compile_error!("enable exactly one network feature: mainnet, testnet or localnet");
#[cfg(not(any(feature = "mainnet", feature = "testnet", feature = "localnet")))]
pub const CORE_BRIDGE_ID: Pubkey = [0; 32];
#[cfg(not(any(feature = "mainnet", feature = "testnet", feature = "localnet")))]
pub const CORE_BRIDGE_ID_BASE58: &str = "";

#[cfg(any(
    all(feature = "mainnet", feature = "testnet"),
    all(feature = "mainnet", feature = "localnet"),
    all(feature = "testnet", feature = "localnet"),
))]
compile_error!("network features mainnet, testnet and localnet are mutually exclusive");

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

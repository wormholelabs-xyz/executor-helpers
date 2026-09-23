//! Mollusk tests. Run `just test`: the ELFs must be built for the same
//! network feature as this test binary.

use core::str::FromStr;

use mollusk_svm::{
    program::{create_program_account_loader_v3, keyed_account_for_system_program},
    result::ProgramResult,
    Mollusk,
};
use post_vaa_relay::{
    ACCOUNT_COUNT, CORE_BRIDGE_ID, CORE_BRIDGE_ID_BASE58, CPI_ACCOUNT_COUNT, POST_VAA_DISCRIMINATOR,
};
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_instruction_error::InstructionError;
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;

// Mirrors the stub's return data layout.
const ACCOUNT_RECORD_LEN: usize = 33;
const FLAG_WRITABLE: u8 = 1;
const FLAG_SIGNER: u8 = 2;

const RELAY_ID: Pubkey = Pubkey::new_from_array([7u8; 32]);

fn core_bridge_id() -> Pubkey {
    Pubkey::new_from_array(CORE_BRIDGE_ID)
}

/// Indexes into the relay account list.
const CORE_BRIDGE_ACCOUNT_INDEX: usize = 0;
const MESSAGE_ACCOUNT_INDEX: usize = 4;
const PAYER_ACCOUNT_INDEX: usize = 5;

struct Fixture {
    mollusk: Mollusk,
    keys: [Pubkey; ACCOUNT_COUNT],
    accounts: Vec<(Pubkey, Account)>,
}

fn fixture() -> Fixture {
    let mut mollusk = Mollusk::new(&RELAY_ID, "post_vaa_relay");
    mollusk.add_program(&core_bridge_id(), "post_vaa_relay_cpi_stub");

    let bridge_owned = || Account {
        lamports: 1_000_000,
        owner: core_bridge_id(),
        ..Account::default()
    };
    let (clock, clock_account) = mollusk.sysvars.keyed_account_for_clock_sysvar();
    let (rent, rent_account) = mollusk.sysvars.keyed_account_for_rent_sysvar();
    let (system_program, system_account) = keyed_account_for_system_program();

    let accounts = vec![
        (
            core_bridge_id(),
            create_program_account_loader_v3(&core_bridge_id()),
        ),
        (Pubkey::new_unique(), bridge_owned()),
        (Pubkey::new_unique(), bridge_owned()),
        (Pubkey::new_unique(), bridge_owned()),
        (Pubkey::new_unique(), Account::default()),
        (
            Pubkey::new_unique(),
            Account {
                lamports: 1_000_000_000,
                ..Account::default()
            },
        ),
        (clock, clock_account),
        (rent, rent_account),
        (system_program, system_account),
    ];
    assert_eq!(accounts.len(), ACCOUNT_COUNT);
    let keys: [Pubkey; ACCOUNT_COUNT] = core::array::from_fn(|i| accounts[i].0);

    Fixture {
        mollusk,
        keys,
        accounts,
    }
}

/// Metas as the outer transaction passes them: `signature_set` writable for
/// `verify_signatures`, everything else as `getPostVaaAccounts` declares.
fn relay_metas(keys: &[Pubkey; ACCOUNT_COUNT]) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(keys[0], false),
        AccountMeta::new_readonly(keys[1], false),
        AccountMeta::new_readonly(keys[2], false),
        AccountMeta::new(keys[3], false),
        AccountMeta::new(keys[4], false),
        AccountMeta::new(keys[5], true),
        AccountMeta::new_readonly(keys[6], false),
        AccountMeta::new_readonly(keys[7], false),
        AccountMeta::new_readonly(keys[8], false),
    ]
}

/// Discriminator plus a Borsh `PostVAAData` with a 3-byte payload.
fn post_vaa_data() -> Vec<u8> {
    let mut data = vec![POST_VAA_DISCRIMINATOR];
    data.push(1); // version
    data.extend_from_slice(&4u32.to_le_bytes()); // guardian_set_index
    data.extend_from_slice(&1_700_000_000u32.to_le_bytes()); // timestamp
    data.extend_from_slice(&42u32.to_le_bytes()); // nonce
    data.extend_from_slice(&2u16.to_le_bytes()); // emitter_chain
    data.extend_from_slice(&[0xAB; 32]); // emitter_address
    data.extend_from_slice(&7u64.to_le_bytes()); // sequence
    data.push(1); // consistency_level
    data.extend_from_slice(&3u32.to_le_bytes()); // payload length
    data.extend_from_slice(&[1, 2, 3]);
    assert_eq!(data.len(), 64);
    data
}

#[test]
fn core_bridge_ids_match_base58() {
    use post_vaa_relay::core_bridge_ids::*;
    let cases: [(&str, [u8; 32], &str); 3] = [
        ("mainnet", MAINNET, MAINNET_BASE58),
        ("testnet", TESTNET, TESTNET_BASE58),
        ("localnet", LOCALNET, LOCALNET_BASE58),
    ];
    for (name, bytes, base58) in cases {
        let decoded = Pubkey::from_str(base58).expect(name);
        assert_eq!(decoded.to_bytes(), bytes, "{name}: bytes match base58");
        assert_eq!(
            Pubkey::new_from_array(bytes).to_string(),
            base58,
            "{name}: base58 round trip"
        );
        assert_ne!(bytes, [0u8; 32], "{name}: non-zero");
    }
    assert_ne!(MAINNET, TESTNET);
    assert_ne!(MAINNET, LOCALNET);
    assert_ne!(TESTNET, LOCALNET);
}

#[test]
fn selected_core_bridge_id_matches_base58() {
    let decoded = Pubkey::from_str(CORE_BRIDGE_ID_BASE58).expect("valid base58");
    assert_eq!(decoded.to_bytes(), CORE_BRIDGE_ID);
    assert_ne!(CORE_BRIDGE_ID, [0u8; 32]);
}

#[test]
fn forwards_post_vaa_with_signature_set_readonly() {
    let f = fixture();
    let data = post_vaa_data();
    let ix = Instruction::new_with_bytes(RELAY_ID, &data, relay_metas(&f.keys));

    let result = f.mollusk.process_instruction(&ix, &f.accounts);

    assert_eq!(result.program_result, ProgramResult::Success);
    assert_eq!(
        result.return_data.len(),
        CPI_ACCOUNT_COUNT * ACCOUNT_RECORD_LEN + data.len()
    );

    let expected_flags: [u8; CPI_ACCOUNT_COUNT] = [
        0,                           // guardian_set
        0,                           // bridge_info
        0,                           // signature_set: downgraded
        FLAG_WRITABLE,               // message
        FLAG_WRITABLE | FLAG_SIGNER, // payer
        0,                           // clock
        0,                           // rent
        0,                           // system_program
    ];
    for (i, expected_flag) in expected_flags.iter().enumerate() {
        let record = &result.return_data[i * ACCOUNT_RECORD_LEN..(i + 1) * ACCOUNT_RECORD_LEN];
        let expected_key = f.keys[i + 1];
        assert_eq!(&record[..32], expected_key.as_ref(), "key at CPI slot {i}");
        assert_eq!(record[32], *expected_flag, "flags at CPI slot {i}");
    }
    assert_eq!(
        &result.return_data[CPI_ACCOUNT_COUNT * ACCOUNT_RECORD_LEN..],
        &data[..],
        "instruction data forwarded verbatim"
    );
}

enum Expected {
    Program(ProgramError),
    Runtime(InstructionError),
}

struct Case {
    name: &'static str,
    mutate: fn(&mut Fixture, &mut Vec<AccountMeta>, &mut Vec<u8>),
    expected: Expected,
}

#[test]
fn rejects_invalid_inputs() {
    let cases = [
        Case {
            name: "wrong core bridge id",
            mutate: |f, metas, _| {
                let impostor = Pubkey::new_unique();
                f.accounts[CORE_BRIDGE_ACCOUNT_INDEX] =
                    (impostor, create_program_account_loader_v3(&impostor));
                metas[CORE_BRIDGE_ACCOUNT_INDEX].pubkey = impostor;
            },
            expected: Expected::Program(ProgramError::IncorrectProgramId),
        },
        Case {
            name: "core bridge account not executable",
            mutate: |f, _, _| {
                f.accounts[CORE_BRIDGE_ACCOUNT_INDEX].1.executable = false;
            },
            expected: Expected::Program(ProgramError::InvalidAccountData),
        },
        Case {
            name: "eight accounts",
            mutate: |_, metas, _| {
                metas.pop();
            },
            expected: Expected::Program(ProgramError::NotEnoughAccountKeys),
        },
        Case {
            name: "ten accounts",
            mutate: |f, metas, _| {
                metas.push(AccountMeta::new_readonly(
                    f.keys[PAYER_ACCOUNT_INDEX],
                    false,
                ));
            },
            expected: Expected::Program(ProgramError::InvalidArgument),
        },
        Case {
            name: "verify_signatures discriminator",
            mutate: |_, _, data| data[0] = 7,
            expected: Expected::Program(ProgramError::InvalidInstructionData),
        },
        Case {
            name: "empty data",
            mutate: |_, _, data| data.clear(),
            expected: Expected::Program(ProgramError::InvalidInstructionData),
        },
        Case {
            name: "message passed read-only",
            mutate: |_, metas, _| metas[MESSAGE_ACCOUNT_INDEX].is_writable = false,
            expected: Expected::Runtime(InstructionError::PrivilegeEscalation),
        },
        Case {
            name: "payer not a signer",
            mutate: |_, metas, _| metas[PAYER_ACCOUNT_INDEX].is_signer = false,
            expected: Expected::Runtime(InstructionError::PrivilegeEscalation),
        },
    ];
    assert!(!cases.is_empty());

    for case in cases {
        let mut f = fixture();
        let mut metas = relay_metas(&f.keys);
        let mut data = post_vaa_data();
        (case.mutate)(&mut f, &mut metas, &mut data);
        let ix = Instruction::new_with_bytes(RELAY_ID, &data, metas);

        let result = f.mollusk.process_instruction(&ix, &f.accounts);

        match case.expected {
            Expected::Program(err) => assert_eq!(
                result.program_result,
                ProgramResult::Failure(err),
                "{}",
                case.name
            ),
            Expected::Runtime(err) => {
                assert_eq!(result.raw_result, Err(err), "{}", case.name);
            }
        }
        assert!(
            result.return_data.is_empty(),
            "{}: no CPI reached the stub",
            case.name
        );
    }
}

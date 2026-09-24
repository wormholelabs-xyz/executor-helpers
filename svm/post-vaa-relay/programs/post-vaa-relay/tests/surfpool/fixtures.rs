//! Network selection and fixture loading. Small fixtures are committed under
//! `tests/fixtures/<net>/`; the core bridge programdata is fetched into
//! `target/fixtures/<net>/` by `scripts/fetch-fixtures.ts`.

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use base64::Engine;
use serde_json::Value;
use sha2::{Digest, Sha256};
use solana_pubkey::Pubkey;

use super::{
    core_bridge::{self, ParsedVaa},
    rpc::hex_encode,
    surfnet::Account,
};

/// Fixture sets under `tests/fixtures/`, with the SHA-256 of each set's core
/// bridge ELF (programdata bytes `[45..]`). Update a pin on purpose, after a
/// review of the upgrade.
const FIXTURE_SETS: [(&str, &str); 2] = [
    (
        "mainnet",
        "a9ce97f30d37f905ee55c7e72d3777c9fc6c5108a501d9e790526a9ffb3a5af1",
    ),
    (
        "testnet",
        "dc269ff28697ef331426f512e784c3193df05f9c1671d5ee16f2a09c38aee224",
    ),
];

/// The fixture set whose `core_bridge_program.json` pubkey equals the
/// compiled-in `CORE_BRIDGE_ADDRESS`. Panics when no set matches.
pub fn network() -> &'static str {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for (name, _) in FIXTURE_SETS {
        let path = root.join(name).join("core_bridge_program.json");
        let fixture = account_fixture_from_json(&read_json(&path), &path);
        if fixture.pubkey == core_bridge_id() {
            return name;
        }
    }
    panic!(
        "no fixture set under tests/fixtures matches CORE_BRIDGE_ADDRESS={}; sets exist for {:?}",
        post_vaa_relay::CORE_BRIDGE_ID_BASE58,
        FIXTURE_SETS.map(|(name, _)| name)
    );
}

fn elf_sha256_pin(network: &str) -> &'static str {
    FIXTURE_SETS
        .iter()
        .find(|(name, _)| *name == network)
        .map(|(_, sha)| *sha)
        .unwrap_or_else(|| panic!("no ELF pin for fixture set {network}"))
}

/// Upgradeable loader programdata header: `UpgradeableLoaderState::ProgramData`.
pub const PROGRAMDATA_HEADER_LEN: usize = 45;

/// Deterministic relay program id for the surfnet.
pub fn relay_program_id() -> Pubkey {
    Pubkey::new_from_array([0x51; 32])
}

pub fn core_bridge_id() -> Pubkey {
    Pubkey::new_from_array(post_vaa_relay::CORE_BRIDGE_ID)
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(network())
}

fn workspace_target_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target")
}

#[derive(Clone, Debug)]
pub struct AccountFixture {
    pub pubkey: Pubkey,
    pub slot: u64,
    pub account: Account,
}

fn read_json(path: &Path) -> Value {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn account_fixture_from_json(value: &Value, path: &Path) -> AccountFixture {
    let field = |k: &str| {
        value[k]
            .as_str()
            .unwrap_or_else(|| panic!("{} lacks {k}", path.display()))
    };
    AccountFixture {
        pubkey: Pubkey::from_str(field("pubkey")).expect("fixture pubkey"),
        slot: value["slot"].as_u64().expect("fixture slot"),
        account: Account {
            lamports: value["lamports"].as_u64().expect("fixture lamports"),
            owner: Pubkey::from_str(field("owner")).expect("fixture owner"),
            executable: value["executable"].as_bool().expect("fixture executable"),
            data: base64::engine::general_purpose::STANDARD
                .decode(field("data_base64"))
                .expect("fixture data base64"),
        },
    }
}

pub fn load_account_fixture(name: &str) -> AccountFixture {
    let path = fixtures_dir().join(name);
    account_fixture_from_json(&read_json(&path), &path)
}

pub fn load_vaa_fixture() -> (String, ParsedVaa) {
    let path = fixtures_dir().join("vaa.json");
    let value = read_json(&path);
    let id = value["id"].as_str().expect("vaa id").to_string();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(value["vaa_base64"].as_str().expect("vaa_base64"))
        .expect("vaa base64");
    let vaa = core_bridge::parse_vaa(&bytes).unwrap_or_else(|e| panic!("parse VAA {id}: {e}"));
    (id, vaa)
}

/// Core bridge ELF sliced from the programdata fixture in `target/fixtures`,
/// checked against its pin in `FIXTURE_SETS`.
pub fn load_core_bridge_elf() -> Vec<u8> {
    let network = network();
    let path = workspace_target_dir()
        .join("fixtures")
        .join(network)
        .join("core_bridge_programdata.json");
    assert!(
        path.exists(),
        "{} missing: run `just fetch-fixtures {network}`",
        path.display()
    );
    let fixture = account_fixture_from_json(&read_json(&path), &path);
    assert!(
        fixture.account.data.len() > PROGRAMDATA_HEADER_LEN,
        "programdata too short"
    );
    let elf = fixture.account.data[PROGRAMDATA_HEADER_LEN..].to_vec();
    let digest = hex_encode(&Sha256::digest(&elf));
    assert_eq!(
        digest,
        elf_sha256_pin(network),
        "core bridge ELF for {network} changed; review the upgrade, then update the pin"
    );
    elf
}

pub fn load_relay_elf() -> Vec<u8> {
    let path = workspace_target_dir().join("deploy/post_vaa_relay.so");
    std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "{}: {e}. Run `just build {}` so the ELF matches this test's CORE_BRIDGE_ADDRESS",
            path.display(),
            network()
        )
    })
}

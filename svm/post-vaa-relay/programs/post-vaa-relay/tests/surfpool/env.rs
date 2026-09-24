//! A seeded surfnet plus everything a test needs to build the transactions.

use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

use super::{
    core_bridge::{self, GuardianSet, ParsedVaa},
    fixtures::{
        core_bridge_id, load_account_fixture, load_core_bridge_elf, load_relay_elf,
        load_vaa_fixture, relay_program_id,
    },
    surfnet::{start_surfpool, Surfpool},
    tx_v1,
};

/// Budget carried in the v1 transaction config. Measured on surfpool 1.6.0:
/// 94,515 CU for the 13-signature mainnet fixture, 84,494 CU for devnet.
/// The core bridge program data alone is about 1 MiB of loaded account data.
pub const BUDGET: tx_v1::Budget = tx_v1::Budget {
    compute_unit_limit: 200_000,
    loaded_accounts_data_size_limit: 8 * 1024 * 1024,
};
pub const PAYER_AIRDROP_LAMPORTS: u64 = 10_000_000_000;

/// A seeded surfnet plus everything a test needs to build the transactions.
pub struct Env {
    pub surfpool: Surfpool,
    pub payer: Keypair,
    pub core_bridge: Pubkey,
    pub relay: Pubkey,
    pub bridge_config: Pubkey,
    pub guardian_set_pubkey: Pubkey,
    pub guardian_set: GuardianSet,
    pub vaa_id: String,
    pub vaa: ParsedVaa,
}

/// Start surfpool, seed the core bridge program and state, deploy the relay,
/// fund the payer. Fixtures must be consistent with each other.
pub fn setup() -> Env {
    let core_bridge = core_bridge_id();
    let bridge = load_account_fixture("bridge.json");
    assert_eq!(
        bridge.pubkey,
        core_bridge::bridge_config_pda(&core_bridge),
        "bridge PDA"
    );
    let guardian_set_index =
        core_bridge::decode_bridge_guardian_set_index(&bridge.account.data).expect("bridge data");
    let guardian_set_fixture =
        load_account_fixture(&format!("guardian_set_{guardian_set_index}.json"));
    assert_eq!(
        guardian_set_fixture.pubkey,
        core_bridge::guardian_set_pda(&core_bridge, guardian_set_index),
        "guardian set PDA"
    );
    let guardian_set =
        core_bridge::decode_guardian_set(&guardian_set_fixture.account.data).expect("guardian set");
    assert_eq!(guardian_set.index, guardian_set_index);
    assert!(!guardian_set.keys.is_empty());

    let (vaa_id, vaa) = load_vaa_fixture();
    assert_eq!(
        vaa.guardian_set_index, guardian_set_index,
        "VAA signed by the seeded set"
    );
    assert!(
        vaa.signatures.len() >= core_bridge::quorum(guardian_set.keys.len()),
        "fixture VAA reaches quorum"
    );

    let surfpool = start_surfpool();
    surfpool.write_program(&core_bridge, &load_core_bridge_elf());
    surfpool.set_account(&bridge.pubkey, &bridge.account);
    surfpool.set_account(&guardian_set_fixture.pubkey, &guardian_set_fixture.account);
    let relay = relay_program_id();
    surfpool.write_program(&relay, &load_relay_elf());

    let payer = Keypair::new();
    surfpool.airdrop(&payer.pubkey(), PAYER_AIRDROP_LAMPORTS);

    Env {
        surfpool,
        payer,
        core_bridge,
        relay,
        bridge_config: bridge.pubkey,
        guardian_set_pubkey: guardian_set_fixture.pubkey,
        guardian_set,
        vaa_id,
        vaa,
    }
}

impl Env {
    /// Signatures paired with the guardian's Ethereum address, for the secp instruction.
    pub fn signatures_with_addresses(&self) -> Vec<(core_bridge::GuardianSignature, [u8; 20])> {
        self.vaa
            .signatures
            .iter()
            .map(|s| {
                let key = self
                    .guardian_set
                    .keys
                    .get(s.guardian_index as usize)
                    .unwrap_or_else(|| panic!("guardian index {} out of range", s.guardian_index));
                (s.clone(), *key)
            })
            .collect()
    }

    pub fn posted_vaa_pubkey(&self) -> Pubkey {
        core_bridge::posted_vaa_pda(&self.core_bridge, &self.vaa.body_hash)
    }

    pub fn post_vaa_accounts(&self, signature_set: &Pubkey) -> core_bridge::PostVaaAccounts {
        core_bridge::PostVaaAccounts {
            guardian_set: self.guardian_set_pubkey,
            bridge_config: self.bridge_config,
            signature_set: *signature_set,
            message: self.posted_vaa_pubkey(),
            payer: self.payer.pubkey(),
        }
    }
}

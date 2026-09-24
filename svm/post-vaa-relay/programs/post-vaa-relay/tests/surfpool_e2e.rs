//! End-to-end proof on a surfpool replica of the network's core bridge state:
//! the relay lets `verify_signatures` (needs `signature_set` writable) and
//! `post_vaa` (declares it read-only) share one transaction. Every
//! transaction is a Solana v1 transaction (SIMD-0385).
//!
//! Run: `just surfpool-test <mainnet|testnet>`. The `#[ignore]` tests need
//! surfpool >= 1.6.0, the relay ELF in `target/deploy`, and the fetched
//! programdata fixture in `target/fixtures/<net>`.

mod surfpool;

use serde_json::{json, Value};
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_sdk_ids::system_program;
use solana_signer::Signer;
use surfpool::{
    core_bridge::{self, quorum},
    tx_v1, Env, BUDGET, NETWORK,
};

/// Relay account index that holds the core bridge program.
const RELAY_CORE_BRIDGE_ACCOUNT_INDEX: usize = 0;
/// Index of the relay (or direct `post_vaa`) instruction in the transaction.
const POST_VAA_INSTRUCTION_INDEX: u64 = 2;

/// `[secp, verify_signatures, post_vaa via relay]` for the fixture VAA.
fn relay_flow(env: &Env, signature_set: &Pubkey) -> Vec<Instruction> {
    let signatures = env.signatures_with_addresses();
    let signers = core_bridge::verify_signatures_signers(&env.vaa.signatures);
    vec![
        core_bridge::secp256k1_instruction(&signatures, &env.vaa.body_hash, 0),
        core_bridge::verify_signatures_instruction(
            &env.core_bridge,
            &env.payer.pubkey(),
            &env.guardian_set_pubkey,
            signature_set,
            signers,
        ),
        core_bridge::relay_instruction(
            &env.relay,
            &env.core_bridge,
            &env.post_vaa_accounts(signature_set),
            &env.vaa,
        ),
    ]
}

fn logs(meta: &Value) -> Vec<String> {
    meta["logMessages"]
        .as_array()
        .expect("logMessages")
        .iter()
        .map(|l| l.as_str().expect("log line").to_string())
        .collect()
}

fn assert_log_order(logs: &[String], expected: &[String]) {
    let mut cursor = 0;
    for line in expected {
        let found = logs[cursor..]
            .iter()
            .position(|l| l == line)
            .unwrap_or_else(|| panic!("log {line:?} not found after index {cursor} in {logs:#?}"));
        cursor += found + 1;
    }
}

// ------------------------------------------------------------ pure tests

#[test]
fn fixture_vaa_matches_seeded_guardian_set() {
    let bridge = surfpool::load_account_fixture("bridge.json");
    let index = core_bridge::decode_bridge_guardian_set_index(&bridge.account.data).unwrap();
    let set_fixture = surfpool::load_account_fixture(&format!("guardian_set_{index}.json"));
    let set = core_bridge::decode_guardian_set(&set_fixture.account.data).unwrap();
    let (id, vaa) = surfpool::load_vaa_fixture();

    assert_eq!(set.index, index);
    assert_eq!(set.expiration_time, 0, "guardian set {index} never expires");
    assert_eq!(vaa.guardian_set_index, index, "VAA {id}");
    assert!(vaa.signatures.len() >= quorum(set.keys.len()));
    assert!(vaa.signatures.len() <= set.keys.len());
    for s in &vaa.signatures {
        assert!((s.guardian_index as usize) < set.keys.len());
    }
    let mut indices: Vec<u8> = vaa.signatures.iter().map(|s| s.guardian_index).collect();
    indices.dedup();
    assert_eq!(indices.len(), vaa.signatures.len(), "no duplicate signers");
}

#[test]
fn pdas_match_fixture_pubkeys() {
    let core = surfpool::core_bridge_id();
    let bridge = surfpool::load_account_fixture("bridge.json");
    assert_eq!(core_bridge::bridge_config_pda(&core), bridge.pubkey);
    assert_eq!(bridge.account.owner, core);
    let index = core_bridge::decode_bridge_guardian_set_index(&bridge.account.data).unwrap();
    let set = surfpool::load_account_fixture(&format!("guardian_set_{index}.json"));
    assert_eq!(core_bridge::guardian_set_pda(&core, index), set.pubkey);
    assert_eq!(set.account.owner, core);
    let program = surfpool::load_account_fixture("core_bridge_program.json");
    assert_eq!(program.pubkey, core);
    assert!(program.account.executable);
}

// ------------------------------------------------------------ surfpool tests

#[test]
#[ignore = "needs surfpool >= 1.6.0 and fetched fixtures; run via `just surfpool-test <net>`"]
fn posts_vaa_through_relay_in_one_v1_transaction() {
    let env = surfpool::setup();
    eprintln!(
        "[e2e] {NETWORK} VAA {} ({} signatures)",
        env.vaa_id,
        env.vaa.signatures.len()
    );

    // Precompile probe: one real signature, alone.
    let probe = tx_v1::build(
        &env.payer,
        &[],
        &[core_bridge::secp256k1_instruction(
            &env.signatures_with_addresses()[..1],
            &env.vaa.body_hash,
            0,
        )],
        env.surfpool.latest_blockhash(),
        BUDGET,
    );
    let probe_sig = env.surfpool.send_and_confirm(&probe.bytes);
    let probe_tx = env.surfpool.get_transaction(&probe_sig);
    assert!(
        probe_tx["meta"]["err"].is_null(),
        "secp256k1 precompile probe: {probe_tx}"
    );

    // The proof: one v1 transaction.
    let signature_set = Keypair::new();
    let instructions = relay_flow(&env, &signature_set.pubkey());
    let tx = tx_v1::build(
        &env.payer,
        &[&signature_set],
        &instructions,
        env.surfpool.latest_blockhash(),
        BUDGET,
    );
    eprintln!("[e2e] transaction size {} bytes", tx.bytes.len());
    assert!(
        env.surfpool.get_account(&env.posted_vaa_pubkey()).is_none(),
        "PostedVAA absent before"
    );

    let signature = env.surfpool.send_and_confirm(&tx.bytes);
    let confirmed = env.surfpool.get_transaction(&signature);
    assert_eq!(confirmed["version"], json!(1), "transaction version");
    let meta = &confirmed["meta"];
    assert!(
        meta["err"].is_null(),
        "transaction failed: {:#?}",
        logs(meta)
    );
    let units = meta["computeUnitsConsumed"]
        .as_u64()
        .expect("computeUnitsConsumed");
    eprintln!("[e2e] compute units consumed {units}");
    assert!(units <= BUDGET.compute_unit_limit as u64);
    assert_log_order(
        &logs(meta),
        &[
            format!("Program {} invoke [1]", env.relay),
            format!("Program {} invoke [2]", env.core_bridge),
            format!("Program {} success", env.core_bridge),
            format!("Program {} success", env.relay),
        ],
    );

    // PostedVAA contents equal the VAA.
    let posted_account = env
        .surfpool
        .get_account(&env.posted_vaa_pubkey())
        .expect("PostedVAA exists");
    assert_eq!(posted_account.owner, env.core_bridge);
    assert!(posted_account.lamports > 0);
    let posted = core_bridge::decode_posted_vaa(&posted_account.data).expect("PostedVAA layout");
    assert_eq!(posted.vaa_version, env.vaa.version);
    assert_eq!(posted.consistency_level, env.vaa.consistency_level);
    assert_eq!(posted.vaa_time, env.vaa.timestamp);
    assert_eq!(posted.vaa_signature_account, signature_set.pubkey());
    assert_eq!(posted.nonce, env.vaa.nonce);
    assert_eq!(posted.sequence, env.vaa.sequence);
    assert_eq!(posted.emitter_chain, env.vaa.emitter_chain);
    assert_eq!(posted.emitter_address, env.vaa.emitter_address);
    assert_eq!(posted.payload, env.vaa.payload);
    assert!(posted.submission_time > 0);

    // SignatureSet marks exactly the fixture's signers.
    let set_account = env
        .surfpool
        .get_account(&signature_set.pubkey())
        .expect("SignatureSet exists");
    assert_eq!(set_account.owner, env.core_bridge);
    let set = core_bridge::decode_signature_set(&set_account.data).expect("SignatureSet layout");
    assert_eq!(set.signatures.len(), env.guardian_set.keys.len());
    assert_eq!(set.hash, env.vaa.body_hash);
    assert_eq!(set.guardian_set_index, env.guardian_set.index);
    for (i, signed) in set.signatures.iter().enumerate() {
        let expected = env
            .vaa
            .signatures
            .iter()
            .any(|s| s.guardian_index as usize == i);
        assert_eq!(*signed, expected, "signature flag for guardian {i}");
    }
}

struct NegativeCase {
    name: &'static str,
    instructions: Vec<Instruction>,
    expected_err: Value,
    expected_log_fragment: &'static str,
}

#[test]
#[ignore = "needs surfpool >= 1.6.0 and fetched fixtures; run via `just surfpool-test <net>`"]
fn rejects_invalid_inputs() {
    let env = surfpool::setup();
    let signature_set = Keypair::new();
    let accounts = env.post_vaa_accounts(&signature_set.pubkey());
    let baseline = relay_flow(&env, &signature_set.pubkey());
    let custom_zero = json!({"InstructionError": [POST_VAA_INSTRUCTION_INDEX, {"Custom": 0}]});

    let cases = [
        NegativeCase {
            name:
                "direct post_vaa without the relay: signature_set is writable at transaction level",
            instructions: {
                let mut ixs = baseline.clone();
                ixs[2] =
                    core_bridge::direct_post_vaa_instruction(&env.core_bridge, &accounts, &env.vaa);
                ixs
            },
            expected_err: custom_zero.clone(),
            expected_log_fragment: "Error: InvalidMutability(",
        },
        NegativeCase {
            name: "relay account 0 is not the core bridge",
            instructions: {
                let mut ixs = baseline.clone();
                ixs[2].accounts[RELAY_CORE_BRIDGE_ACCOUNT_INDEX].pubkey = system_program::id();
                ixs
            },
            expected_err: json!({"InstructionError": [POST_VAA_INSTRUCTION_INDEX, "IncorrectProgramId"]}),
            expected_log_fragment: "incorrect program id",
        },
    ];
    assert_eq!(cases.len(), 2);

    for case in &cases {
        let tx = tx_v1::build(
            &env.payer,
            &[&signature_set],
            &case.instructions,
            env.surfpool.latest_blockhash(),
            BUDGET,
        );
        let result = env.surfpool.simulate(&tx.bytes);
        assert_eq!(
            result["err"], case.expected_err,
            "{}: {result:#?}",
            case.name
        );
        let logs: Vec<String> = result["logs"]
            .as_array()
            .unwrap_or_else(|| panic!("{}: no logs in {result:#?}", case.name))
            .iter()
            .map(|l| l.as_str().unwrap_or_default().to_string())
            .collect();
        assert!(
            logs.iter().any(|l| l.contains(case.expected_log_fragment)),
            "{}: expected log containing {:?} in {logs:#?}",
            case.name,
            case.expected_log_fragment
        );
        assert!(
            env.surfpool.get_account(&env.posted_vaa_pubkey()).is_none(),
            "{}: PostedVAA must not exist",
            case.name
        );
    }
}

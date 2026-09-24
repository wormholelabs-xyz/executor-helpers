//! Pure builders and decoders for the Wormhole core bridge. Layouts follow
//! `wormhole/solana/bridge/program/src` (Solitaire, Borsh little-endian).

use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use solana_sdk_ids::{secp256k1_program, system_program, sysvar};

pub const MAX_GUARDIANS: usize = 19;
pub const VERIFY_SIGNATURES_DISCRIMINATOR: u8 = 7;
/// One VAA signature record: guardian index, r||s, recovery id.
pub const VAA_SIGNATURE_LEN: usize = 66;
/// Body prefix before the payload: timestamp, nonce, chain, emitter, sequence, consistency.
pub const VAA_BODY_PREFIX_LEN: usize = 4 + 4 + 2 + 32 + 8 + 1;
pub const POSTED_VAA_MAGIC: &[u8; 3] = b"vaa";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardianSignature {
    pub guardian_index: u8,
    pub signature: [u8; 64],
    pub recovery_id: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedVaa {
    pub version: u8,
    pub guardian_set_index: u32,
    pub signatures: Vec<GuardianSignature>,
    pub body: Vec<u8>,
    pub timestamp: u32,
    pub nonce: u32,
    pub emitter_chain: u16,
    pub emitter_address: [u8; 32],
    pub sequence: u64,
    pub consistency_level: u8,
    pub payload: Vec<u8>,
    /// `keccak256(body)`: the message the secp256k1 instruction carries and
    /// the `PostedVAA` PDA seed. Guardians sign `keccak256` of this hash.
    pub body_hash: [u8; 32],
}

pub fn parse_vaa(bytes: &[u8]) -> Result<ParsedVaa, String> {
    if bytes.len() < 6 {
        return Err(format!("VAA too short: {} bytes", bytes.len()));
    }
    let version = bytes[0];
    if version != 1 {
        return Err(format!("unsupported VAA version {version}"));
    }
    let guardian_set_index = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
    let count = bytes[5] as usize;
    if count == 0 || count > MAX_GUARDIANS {
        return Err(format!("signature count {count} out of range"));
    }
    let body_offset = 6 + count * VAA_SIGNATURE_LEN;
    if bytes.len() < body_offset + VAA_BODY_PREFIX_LEN {
        return Err("VAA truncated before body".to_string());
    }
    let mut signatures = Vec::with_capacity(count);
    for i in 0..count {
        let at = 6 + i * VAA_SIGNATURE_LEN;
        let mut signature = [0u8; 64];
        signature.copy_from_slice(&bytes[at + 1..at + 65]);
        signatures.push(GuardianSignature {
            guardian_index: bytes[at],
            signature,
            recovery_id: bytes[at + 65],
        });
    }
    let body = bytes[body_offset..].to_vec();
    let mut emitter_address = [0u8; 32];
    emitter_address.copy_from_slice(&body[10..42]);
    let body_hash = solana_keccak_hasher::hash(&body).to_bytes();
    Ok(ParsedVaa {
        version,
        guardian_set_index,
        signatures,
        timestamp: u32::from_be_bytes(body[0..4].try_into().unwrap()),
        nonce: u32::from_be_bytes(body[4..8].try_into().unwrap()),
        emitter_chain: u16::from_be_bytes([body[8], body[9]]),
        emitter_address,
        sequence: u64::from_be_bytes(body[42..50].try_into().unwrap()),
        consistency_level: body[50],
        payload: body[VAA_BODY_PREFIX_LEN..].to_vec(),
        body_hash,
        body,
    })
}

/// `2/3 + 1` quorum in the core bridge's fixed-point form.
pub fn quorum(guardian_count: usize) -> usize {
    ((guardian_count * 10 / 3) * 2) / 10 + 1
}

pub fn bridge_config_pda(core_bridge: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"Bridge"], core_bridge).0
}

pub fn guardian_set_pda(core_bridge: &Pubkey, index: u32) -> Pubkey {
    Pubkey::find_program_address(&[b"GuardianSet", &index.to_be_bytes()], core_bridge).0
}

pub fn posted_vaa_pda(core_bridge: &Pubkey, body_hash: &[u8; 32]) -> Pubkey {
    Pubkey::find_program_address(&[b"PostedVAA", body_hash], core_bridge).0
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardianSet {
    pub index: u32,
    pub keys: Vec<[u8; 20]>,
    pub creation_time: u32,
    pub expiration_time: u32,
}

pub fn decode_guardian_set(data: &[u8]) -> Result<GuardianSet, String> {
    if data.len() < 8 {
        return Err("guardian set too short".to_string());
    }
    let index = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let count = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
    let end = 8 + count * 20;
    if data.len() != end + 8 {
        return Err(format!("guardian set length {} != {}", data.len(), end + 8));
    }
    let keys = (0..count)
        .map(|i| data[8 + i * 20..8 + (i + 1) * 20].try_into().unwrap())
        .collect();
    Ok(GuardianSet {
        index,
        keys,
        creation_time: u32::from_le_bytes(data[end..end + 4].try_into().unwrap()),
        expiration_time: u32::from_le_bytes(data[end + 4..end + 8].try_into().unwrap()),
    })
}

/// First field of `BridgeData`.
pub fn decode_bridge_guardian_set_index(data: &[u8]) -> Result<u32, String> {
    if data.len() < 4 {
        return Err("bridge config too short".to_string());
    }
    Ok(u32::from_le_bytes(data[0..4].try_into().unwrap()))
}

/// Secp256k1 precompile instruction over `message_hash`, one entry per
/// signature, every offset pointing at `instruction_index`.
/// Layout: `[count u8][11-byte offsets x count][sig 64 + v 1 + eth addr 20 x count][message 32]`.
pub fn secp256k1_instruction(
    signatures: &[(GuardianSignature, [u8; 20])],
    message_hash: &[u8; 32],
    instruction_index: u8,
) -> Instruction {
    const OFFSETS_ENTRY_LEN: usize = 11;
    const SIGNATURE_AND_ADDRESS_LEN: usize = 85;

    let count = signatures.len();
    assert!(
        (1..=MAX_GUARDIANS).contains(&count),
        "signature count {count}"
    );
    let data_offset = 1 + count * OFFSETS_ENTRY_LEN;
    let message_offset = data_offset + count * SIGNATURE_AND_ADDRESS_LEN;

    let mut data = Vec::with_capacity(message_offset + 32);
    data.push(count as u8);
    for i in 0..count {
        let signature_at = (data_offset + SIGNATURE_AND_ADDRESS_LEN * i) as u16;
        data.extend_from_slice(&signature_at.to_le_bytes());
        data.push(instruction_index);
        data.extend_from_slice(&(signature_at + 65).to_le_bytes());
        data.push(instruction_index);
        data.extend_from_slice(&(message_offset as u16).to_le_bytes());
        data.extend_from_slice(&32u16.to_le_bytes());
        data.push(instruction_index);
    }
    for (signature, eth_address) in signatures {
        data.extend_from_slice(&signature.signature);
        data.push(signature.recovery_id);
        data.extend_from_slice(eth_address);
    }
    data.extend_from_slice(message_hash);
    assert_eq!(data.len(), message_offset + 32);

    Instruction {
        program_id: secp256k1_program::id(),
        accounts: vec![],
        data,
    }
}

/// `signers[guardian_index] = position in the secp instruction`, `-1` elsewhere.
pub fn verify_signatures_signers(signatures: &[GuardianSignature]) -> [i8; MAX_GUARDIANS] {
    let mut signers = [-1i8; MAX_GUARDIANS];
    for (position, signature) in signatures.iter().enumerate() {
        let index = signature.guardian_index as usize;
        assert!(index < MAX_GUARDIANS, "guardian index {index}");
        assert_eq!(signers[index], -1, "duplicate guardian index {index}");
        signers[index] = position as i8;
    }
    signers
}

pub fn verify_signatures_instruction(
    core_bridge: &Pubkey,
    payer: &Pubkey,
    guardian_set: &Pubkey,
    signature_set: &Pubkey,
    signers: [i8; MAX_GUARDIANS],
) -> Instruction {
    let mut data = Vec::with_capacity(1 + MAX_GUARDIANS);
    data.push(VERIFY_SIGNATURES_DISCRIMINATOR);
    data.extend(signers.iter().map(|s| *s as u8));
    Instruction {
        program_id: *core_bridge,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(*guardian_set, false),
            AccountMeta::new(*signature_set, true),
            AccountMeta::new_readonly(sysvar::instructions::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}

/// `[POST_VAA_DISCRIMINATOR]` + Borsh `PostVAAData`.
pub fn post_vaa_data(vaa: &ParsedVaa) -> Vec<u8> {
    let mut data = Vec::with_capacity(1 + 1 + 4 + 4 + 4 + 2 + 32 + 8 + 1 + 4 + vaa.payload.len());
    data.push(post_vaa_relay::POST_VAA_DISCRIMINATOR);
    data.push(vaa.version);
    data.extend_from_slice(&vaa.guardian_set_index.to_le_bytes());
    data.extend_from_slice(&vaa.timestamp.to_le_bytes());
    data.extend_from_slice(&vaa.nonce.to_le_bytes());
    data.extend_from_slice(&vaa.emitter_chain.to_le_bytes());
    data.extend_from_slice(&vaa.emitter_address);
    data.extend_from_slice(&vaa.sequence.to_le_bytes());
    data.push(vaa.consistency_level);
    data.extend_from_slice(&(vaa.payload.len() as u32).to_le_bytes());
    data.extend_from_slice(&vaa.payload);
    data
}

pub struct PostVaaAccounts {
    pub guardian_set: Pubkey,
    pub bridge_config: Pubkey,
    pub signature_set: Pubkey,
    pub message: Pubkey,
    pub payer: Pubkey,
}

/// The eight `post_vaa` accounts in `getPostVaaAccounts` order.
fn post_vaa_metas(accounts: &PostVaaAccounts) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(accounts.guardian_set, false),
        AccountMeta::new_readonly(accounts.bridge_config, false),
        AccountMeta::new_readonly(accounts.signature_set, false),
        AccountMeta::new(accounts.message, false),
        AccountMeta::new(accounts.payer, true),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ]
}

/// Relay instruction: the core bridge program first, then the `post_vaa` accounts.
pub fn relay_instruction(
    relay: &Pubkey,
    core_bridge: &Pubkey,
    accounts: &PostVaaAccounts,
    vaa: &ParsedVaa,
) -> Instruction {
    let mut metas = Vec::with_capacity(post_vaa_relay::ACCOUNT_COUNT);
    metas.push(AccountMeta::new_readonly(*core_bridge, false));
    metas.extend(post_vaa_metas(accounts));
    assert_eq!(metas.len(), post_vaa_relay::ACCOUNT_COUNT);
    Instruction {
        program_id: *relay,
        accounts: metas,
        data: post_vaa_data(vaa),
    }
}

/// Core bridge `post_vaa` called without the relay.
pub fn direct_post_vaa_instruction(
    core_bridge: &Pubkey,
    accounts: &PostVaaAccounts,
    vaa: &ParsedVaa,
) -> Instruction {
    Instruction {
        program_id: *core_bridge,
        accounts: post_vaa_metas(accounts),
        data: post_vaa_data(vaa),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostedVaa {
    pub vaa_version: u8,
    pub consistency_level: u8,
    pub vaa_time: u32,
    pub vaa_signature_account: Pubkey,
    pub submission_time: u32,
    pub nonce: u32,
    pub sequence: u64,
    pub emitter_chain: u16,
    pub emitter_address: [u8; 32],
    pub payload: Vec<u8>,
}

/// `b"vaa"` + Borsh `MessageData`.
pub fn decode_posted_vaa(data: &[u8]) -> Result<PostedVaa, String> {
    const FIXED: usize = 3 + 1 + 1 + 4 + 32 + 4 + 4 + 8 + 2 + 32 + 4;
    if data.len() < FIXED {
        return Err(format!("PostedVAA too short: {}", data.len()));
    }
    if &data[0..3] != POSTED_VAA_MAGIC {
        return Err(format!("PostedVAA magic {:?}", &data[0..3]));
    }
    let mut at = 3;
    let mut take = |n: usize| {
        let s = &data[at..at + n];
        at += n;
        s
    };
    let vaa_version = take(1)[0];
    let consistency_level = take(1)[0];
    let vaa_time = u32::from_le_bytes(take(4).try_into().unwrap());
    let vaa_signature_account = Pubkey::new_from_array(take(32).try_into().unwrap());
    let submission_time = u32::from_le_bytes(take(4).try_into().unwrap());
    let nonce = u32::from_le_bytes(take(4).try_into().unwrap());
    let sequence = u64::from_le_bytes(take(8).try_into().unwrap());
    let emitter_chain = u16::from_le_bytes(take(2).try_into().unwrap());
    let emitter_address: [u8; 32] = take(32).try_into().unwrap();
    let payload_len = u32::from_le_bytes(take(4).try_into().unwrap()) as usize;
    if data.len() != FIXED + payload_len {
        return Err(format!(
            "PostedVAA length {} != {}",
            data.len(),
            FIXED + payload_len
        ));
    }
    Ok(PostedVaa {
        vaa_version,
        consistency_level,
        vaa_time,
        vaa_signature_account,
        submission_time,
        nonce,
        sequence,
        emitter_chain,
        emitter_address,
        payload: data[FIXED..].to_vec(),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureSet {
    pub signatures: Vec<bool>,
    pub hash: [u8; 32],
    pub guardian_set_index: u32,
}

/// Borsh `SignatureSetData`: `Vec<bool>`, `[u8; 32]`, `u32`.
pub fn decode_signature_set(data: &[u8]) -> Result<SignatureSet, String> {
    if data.len() < 4 {
        return Err("SignatureSet too short".to_string());
    }
    let count = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
    let expected = 4 + count + 32 + 4;
    if data.len() != expected {
        return Err(format!("SignatureSet length {} != {expected}", data.len()));
    }
    let signatures = data[4..4 + count]
        .iter()
        .map(|b| match b {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(format!("bool byte {other}")),
        })
        .collect::<Result<Vec<bool>, String>>()?;
    Ok(SignatureSet {
        signatures,
        hash: data[4 + count..4 + count + 32].try_into().unwrap(),
        guardian_set_index: u32::from_le_bytes(data[4 + count + 32..].try_into().unwrap()),
    })
}

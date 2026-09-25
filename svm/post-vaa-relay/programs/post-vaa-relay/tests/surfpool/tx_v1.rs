//! Solana transaction v1 (SIMD-0385) build, sign and serialize. Every
//! transaction the harness sends goes through [`build`]. v1 differs from
//! legacy in message layout, `wincode` encoding, and an explicit budget.

use solana_hash::Hash;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::{v1, VersionedMessage};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

pub struct V1Transaction {
    pub transaction: VersionedTransaction,
    /// `wincode` wire bytes, as sent to the RPC.
    pub bytes: Vec<u8>,
}

/// Budget carried in the v1 message config. v1 has no defaults: an absent
/// compute unit limit or loaded accounts data size limit means zero.
#[derive(Clone, Copy, Debug)]
pub struct Budget {
    pub compute_unit_limit: u32,
    pub loaded_accounts_data_size_limit: u32,
}

fn compile(
    payer: &Pubkey,
    instructions: &[Instruction],
    blockhash: Hash,
    budget: Budget,
) -> v1::Message {
    let config = v1::TransactionConfig::empty()
        .with_compute_unit_limit(budget.compute_unit_limit)
        .with_loaded_accounts_data_size_limit(budget.loaded_accounts_data_size_limit);
    let message = v1::Message::try_compile_with_config(payer, instructions, blockhash, config)
        .expect("compile v1 message");
    message.validate().expect("valid v1 message");
    message
}

/// Signed v1 transaction. `payer` signs first; `extra_signers` follow in the
/// order the message requires.
pub fn build(
    payer: &Keypair,
    extra_signers: &[&Keypair],
    instructions: &[Instruction],
    blockhash: Hash,
    budget: Budget,
) -> V1Transaction {
    let message = compile(&payer.pubkey(), instructions, blockhash, budget);
    let mut signers: Vec<&Keypair> = Vec::with_capacity(1 + extra_signers.len());
    signers.push(payer);
    signers.extend_from_slice(extra_signers);
    let transaction = VersionedTransaction::try_new(VersionedMessage::V1(message), &signers)
        .expect("sign v1 transaction");
    let bytes = wincode::serialize(&transaction).expect("serialize v1 transaction");
    assert_eq!(bytes[0], v1::V1_PREFIX, "v1 prefix byte");
    V1Transaction { transaction, bytes }
}

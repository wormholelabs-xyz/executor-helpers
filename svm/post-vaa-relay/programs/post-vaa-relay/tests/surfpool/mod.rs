//! Surfpool end-to-end harness. Every transaction it sends is a Solana v1
//! transaction. Fixtures replicate the network's core bridge state.

#![allow(dead_code)]

pub mod core_bridge;
pub mod env;
pub mod fixtures;
pub mod rpc;
pub mod surfnet;
pub mod tx_v1;

pub use env::{setup, Env, BUDGET};
pub use fixtures::{core_bridge_id, load_account_fixture, load_vaa_fixture, network};

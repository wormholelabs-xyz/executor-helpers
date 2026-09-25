# post-vaa-relay

`post-vaa-relay` is a pinocchio program. It makes one CPI into the Wormhole core bridge `post_vaa` instruction. It passes `signature_set` as read-only. This lets `verify_signatures` and `post_vaa` share one transaction.

The program checks account 0 against a compile-time core bridge program id. The build reads it from the environment variable `CORE_BRIDGE_ADDRESS` (base58). There is no default. A fork sets its own address. The `just` recipes accept a network name and resolve it:

| `network` argument | Core bridge program id | Cluster |
|---|---|---|
| `mainnet` | `worm2ZoG2kUd4vFXhvjh93UUH596ayRfgQ2MgjNMTth` | Solana mainnet-beta |
| `testnet` | `3u8hJUVTA4jH1wYAyUur7FFZVQ8H635K3tSHHF4ssjQ5` | Solana devnet |
| `localnet` | `Bridge1p5gheXUvJ6jGWGeCsgPKgnE3YgdGKRVCMY9o` | Tilt devnet |
| any other value | used as the address | a fork |

## Build

Install `just`. Run the recipes from this directory. Each recipe takes the network as its argument.

```bash
just build testnet
just build mainnet
```

The ELF is `target/deploy/post_vaa_relay.so`.

## Test

The tests use mollusk. They load the relay ELF and a stub core bridge ELF from `target/deploy`. The `test` recipe builds both first.

```bash
just test testnet
just check testnet    # clippy, all targets
```

The host toolchain for this crate is Rust 1.97.1 (`rust-toolchain.toml`). The mollusk dependency tree requires it. The SBF build uses the Solana platform-tools toolchain and is not affected.

## Surfpool end-to-end test

The end-to-end test runs the relay against a replica of the real core bridge. It proves the product claim: `verify_signatures` and `post_vaa` fit in one transaction.

Every transaction in this test is a Solana v1 transaction (SIMD-0385). The mainnet transaction holds the secp256k1 instruction with all 13 guardian signatures, `verify_signatures`, and the relay. That is about 2 KB, above the 1232-byte limit of legacy and v0 transactions.

Requirements:

- surfpool 1.6.0 or newer on `PATH`. Older versions ignore the v1 compute budget. Install or update with `surfpool update -y`.
- The core bridge program data for the network. The recipe fetches it through the RPC proxy into `target/fixtures/<net>/` on the first run. The test checks its SHA-256 against a pinned value.

Run:

```bash
just surfpool-test mainnet
just surfpool-test testnet
```

The recipe builds the relay ELF for the network, then starts an offline surfnet with the `enable_tx_v1` feature. It seeds the core bridge ELF, the bridge config, and the guardian set from fixtures. Then it sends the transactions and reads the results back.

The test checks:

- The transaction is version 1, succeeds, and stays under the compute unit limit in its config.
- The `PostedVAA` account holds the VAA fields byte for byte.
- The `SignatureSet` account marks exactly the guardians that signed.
- The same transaction with `post_vaa` called directly fails with `InvalidMutability`.
- The relay rejects a wrong core bridge id with `IncorrectProgramId`.

Refresh the fixtures when the guardian set or the core bridge program changes:

```bash
just fetch-fixtures mainnet
```

Then update the SHA-256 pin in `programs/post-vaa-relay/tests/surfpool/fixtures.rs` on purpose, after a review of the upgrade.

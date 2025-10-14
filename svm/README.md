
# Building and Deploying SVM Programs

## Prerequisites

Before building the SVM programs, ensure you have the following installed:

### Rust (>= 1.89.0)

Install Rust using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup install 1.89.0
rustup default 1.89.0
```

Verify installation:

```bash
rustc --version
# Should output: rustc 1.89.0 or higher
```

### Solana CLI (2.3.13)

The required Solana CLI version must be installed from the Agave GitHub releases. Package managers may not have this specific version.

1. Download the release from: https://github.com/anza-xyz/agave/releases/tag/v2.3.13
2. Extract and install the binary according to the release instructions
3. Verify installation:

```bash
solana --version
# Should output: solana-cli 2.3.13 (src:5466f459; feat:2142755730, client:Agave)
```

### Anchor CLI (0.32.1)

Install Anchor CLI using avm (Anchor Version Manager):

```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.32.1
avm use 0.32.1
```

Verify installation:

```bash
anchor --version
# Should output: anchor-cli 0.32.1
```

## Building

Navigate to the SVM program directory and build:

```bash
cd relay_cost_protection
anchor build
```

The compiled program will be available in `target/deploy/relay_cost_protection.so`.

## Deployment

### Configure Solana CLI

Set your target cluster (localnet, devnet, testnet, or mainnet-beta):

```bash
# For devnet
solana config set --url https://api.devnet.solana.com

# For mainnet-beta
solana config set --url https://api.mainnet-beta.solana.com
```

Set your wallet:

```bash
solana config set --keypair ~/.config/solana/id.json
```

### Deploy the Program

Deploy using Anchor:

```bash
cd relay_cost_protection
anchor deploy
```

The deployment will output the program ID. Update `Anchor.toml` with the deployed program ID if needed for subsequent deployments or testing.

### Verify Deployment

Check that the program was deployed successfully:

```bash
solana program show <PROGRAM_ID>
```

## Running Tests

The project includes TypeScript tests that can be run against a local validator:

```bash
cd relay_cost_protection
anchor test
```

This command will:
1. Start a local Solana validator
2. Build and deploy the program
3. Run the test suite
4. Stop the validator

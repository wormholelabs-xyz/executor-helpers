
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

## Optimization: Anchor Fallback Instructions

### Overview

The `relay_cost_protection` program currently uses a 1-byte instruction discriminator:

```rust
#[instruction(discriminator = 1)]
pub fn check_balance(ctx: Context<CheckBalance>, min_balance: u64) -> Result<()>
```

This can be further optimized by using Anchor's **fallback instruction** feature to eliminate the discriminator entirely, saving 1 byte of instruction data.

### How Fallback Instructions Work

Anchor supports an undocumented feature where functions that **do not** take a `Context<...>` parameter are treated as fallback functions. When a program has exactly one fallback function, Anchor invokes it directly without requiring a discriminator.

**Key characteristics:**
- Removes the instruction discriminator (saves 1+ bytes)
- Function signature does NOT include `Context<T>`
- Anchor automatically routes to the fallback when no discriminator matches
- Trade-off: IDL generation is not supported for fallback functions

### Implementation Example

Instead of the current implementation:

```rust
#[program]
pub mod relay_cost_protection {
    use super::*;

    #[instruction(discriminator = 1)]
    pub fn check_balance(ctx: Context<CheckBalance>, min_balance: u64) -> Result<()> {
        require!(
            ctx.accounts.payer.lamports() >= min_balance,
            RelayError::InsufficientBalance
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CheckBalance<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
}
```

You could use a fallback function:

```rust
#[program]
pub mod relay_cost_protection {
    use super::*;

    // Fallback function - no Context parameter
    pub fn fallback<'info>(
        _program_id: &Pubkey,
        accounts: &'info [AccountInfo<'info>],
        instruction_data: &[u8],
    ) -> Result<()> {
        // Parse accounts manually
        let account_iter = &mut accounts.iter();
        let payer = next_account_info(account_iter)?;

        // Parse instruction data (just min_balance as u64)
        let min_balance = u64::from_le_bytes(
            instruction_data[0..8].try_into().unwrap()
        );

        // Verify payer is signer
        require!(payer.is_signer, ProgramError::MissingRequiredSignature);

        // Check balance
        require!(
            payer.lamports() >= min_balance,
            RelayError::InsufficientBalance
        );

        Ok(())
    }
}
```

### When to Use Fallback Instructions

**Use fallback instructions when:**
- Program has a single instruction
- Instruction data is simple (no complex serialization needed)
- Minimizing instruction data size is critical
- IDL generation is not required (client can use raw transactions)

**Avoid fallback instructions when:**
- Program has multiple instructions (discriminator needed anyway)
- Complex account validation logic benefits from Anchor's `Context` macros
- IDL generation is important for client integration
- Team prefers Anchor's type safety and automatic account parsing

### References

- [Anchor dispatch implementation](https://github.com/coral-xyz/anchor/blob/master/lang/syn/src/codegen/program/dispatch.rs#L54)
- [Solana Transfer Hook example with fallback](https://solana.com/developers/guides/token-extensions/transfer-hook#fallback-instruction)

### Current Implementation Choice

The `relay_cost_protection` program uses a 1-byte discriminator rather than a fallback instruction because:
1. The 1-byte overhead is minimal for this use case
2. Anchor's `Context` provides better type safety
3. IDL generation enables easier client integration
4. The program is already deployed as immutable on mainnet

For future programs with similar simplicity, the fallback approach could save that additional byte if needed.

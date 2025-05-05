# CCTPv2ReceiveWithGasDropOff EVM Deployments

## Testnet

### May 5, 2025

#### Modifications

The same contract can be reused for v2, just pointed at the v2 MessageTransmitter.

Only the following modification was made.

```solidity
string constant cctpWithExecutorVersion = "CCTPv2ReceiveWithGasDropOff-0.0.1";
```

#### Deployment Commands (Example)

```bash
RPC_URL=https://ethereum-sepolia-rpc.publicnode.com MNEMONIC=<PRIVATE_KEY> EVM_CHAIN_ID=11155111 CIRCLE_MESSAGE_TRANSMITTER_ADDR=0xE737e5cEBEEBa77EFE34D4aa090756590b1CE275 ./sh/deployCCTPv1ReceiveWithGasDropOff.sh
forge verify-contract 0xd940731D42a76083477aA54417d2a0A61E226342 src/CCTPv1ReceiveWithGasDropOff.sol:CCTPv1ReceiveWithGasDropOff --watch --chain-id 11155111 --etherscan-api-key <API_KEY> --constructor-args $(cast abi-encode "constructor(address)" "0xE737e5cEBEEBa77EFE34D4aa090756590b1CE275")
```

#### Chains Deployed

Here are the deployed contract addresses for each chain. The number after the chain name is the Wormhole chain ID configured for the contract.

- Sepolia (10002): [0xd940731D42a76083477aA54417d2a0A61E226342](https://sepolia.etherscan.io/address/0xd940731D42a76083477aA54417d2a0A61E226342)
- Base Sepolia (10004): [0xd940731D42a76083477aA54417d2a0A61E226342](https://sepolia.basescan.org/address/0xd940731D42a76083477aA54417d2a0A61E226342)
- Avalanche Fuji (6): [0xd940731D42a76083477aA54417d2a0A61E226342](https://testnet.snowtrace.io/address/0xd940731D42a76083477aA54417d2a0A61E226342)

# CCTPv1ReceiveWithGasDropOff EVM Deployments

## Testnet

### April, 2025

#### Version Info

Commit Hash:

<!-- cspell:disable -->

```sh
example-ntt-with-executor (main)$ git rev-parse HEAD
926890b37aa55cec28f9796432b62864a426722d
example-ntt-with-executor (main)$
```

<!-- cspell:enable -->

Foundry Version:

<!-- cspell:disable -->

```sh
evm (main)$ forge --version
forge Version: 1.0.0-stable
Commit SHA: e144b82070619b6e10485c38734b4d4d45aebe04
Build Timestamp: 2025-02-13T20:03:31.026474817Z (1739477011)
Build Profile: maxperf
evm (main)$
```

<!-- cspell:enable -->

#### Chains Deployed

Here are the deployed contract addresses for each chain. The number after the chain name is the Wormhole chain ID configured for the contract.

- Sepolia (10002): [0x638DF05cF6e37dE4E69cE5fF32C4Ed8284502Dc4](https://sepolia.etherscan.io/address/0x638df05cf6e37de4e69ce5ff32c4ed8284502dc4)
- Base Sepolia (10004): [0x638DF05cF6e37dE4E69cE5fF32C4Ed8284502Dc4](https://sepolia.basescan.org/address/0x638DF05cF6e37dE4E69cE5fF32C4Ed8284502Dc4)
- Avalanche Fuji (6): [0x3AbE59000a0505979cC591bA975F474B46B65083](https://testnet.snowtrace.io/address/0x3AbE59000a0505979cC591bA975F474B46B65083)

### Bytecode Verification

If you wish to verify that the bytecode built locally matches what is deployed on chain, you can do something like this:

<!-- cspell:disable -->

```
forge verify-bytecode <contract_addr> CCTPv1ReceiveWithGasDropOff --rpc-url <archive_node_rpc> --verifier-api-key <your_etherscan_key>
```

<!-- cspell:enable -->

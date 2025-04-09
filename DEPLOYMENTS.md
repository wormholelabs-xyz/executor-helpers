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

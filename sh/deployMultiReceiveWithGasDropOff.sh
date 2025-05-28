#!/bin/bash

#
# This script deploys the MultiReceiveWithGasDropOff contract.
# Usage: RPC_URL= MNEMONIC= EVM_CHAIN_ID= ./sh/deployMultiReceiveWithGasDropOff.sh

if [ "${RPC_URL}X" == "X" ]; then
  RPC_URL=http://localhost:8545
fi

if [ "${MNEMONIC}X" == "X" ]; then
  MNEMONIC=0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d
fi

if [ "${EVM_CHAIN_ID}X" == "X" ]; then
  EVM_CHAIN_ID=1337
fi

forge script ./script/DeployMultiReceiveWithGasDropOff.s.sol:DeployMultiReceiveWithGasDropOff \
	--sig "run()" \
	--rpc-url "$RPC_URL" \
	--private-key "$MNEMONIC" \
	--broadcast ${FORGE_ARGS}

returnInfo=$(cat ./broadcast/DeployMultiReceiveWithGasDropOff.s.sol/$EVM_CHAIN_ID/run-latest.json)

DEPLOYED_ADDRESS=$(jq -r '.returns.deployedAddress.value' <<< "$returnInfo")
echo "Deployed multi receive with drop off address: $DEPLOYED_ADDRESS"

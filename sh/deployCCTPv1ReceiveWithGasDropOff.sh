#!/bin/bash

#
# This script deploys the CCTPv1ReceiveWithGasDropOff contract.
# Usage: RPC_URL= MNEMONIC= EVM_CHAIN_ID= CIRCLE_MESSAGE_TRANSMITTER_ADDR= ./sh/deployCCTPv1ReceiveWithGasDropOff.sh

[[ -z $CIRCLE_MESSAGE_TRANSMITTER_ADDR ]] && { echo "Missing Circle Token Messenger address"; exit 1; }

if [ "${RPC_URL}X" == "X" ]; then
  RPC_URL=http://localhost:8545
fi

if [ "${MNEMONIC}X" == "X" ]; then
  MNEMONIC=0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d
fi

if [ "${EVM_CHAIN_ID}X" == "X" ]; then
  EVM_CHAIN_ID=1337
fi

forge script ./script/DeployCCTPv1ReceiveWithGasDropOff.s.sol:DeployCCTPv1ReceiveWithGasDropOff \
	--sig "run(address)" $CIRCLE_MESSAGE_TRANSMITTER_ADDR \
	--rpc-url "$RPC_URL" \
	--private-key "$MNEMONIC" \
	--broadcast ${FORGE_ARGS}

returnInfo=$(cat ./broadcast/DeployCCTPv1ReceiveWithGasDropOff.s.sol/$EVM_CHAIN_ID/run-latest.json)

DEPLOYED_ADDRESS=$(jq -r '.returns.deployedAddress.value' <<< "$returnInfo")
echo "Deployed CCTP receive with drop off address: $DEPLOYED_ADDRESS"

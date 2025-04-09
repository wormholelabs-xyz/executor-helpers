// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {CCTPv1ReceiveWithGasDropOff, cctpWithExecutorVersion} from "../src/CCTPv1ReceiveWithGasDropOff.sol";
import "forge-std/Script.sol";

// DeployCCTPv1ReceiveWithGasDropOff is a forge script to deploy the CCTPv1ReceiveWithGasDropOff contract. Use ./sh/deployCCTPv1ReceiveWithGasDropOff.sh to invoke this.
//    EVM_CHAIN_ID= MNEMONIC= CIRCLE_MESSAGE_TRANSMITTER= ./sh/deployCCTPv1ReceiveWithGasDropOff.sh
contract DeployCCTPv1ReceiveWithGasDropOff is Script {
    function test() public {} // Exclude this from coverage report.

    function dryRun(address circleMessageTransmitter) public {
        _deploy(circleMessageTransmitter);
    }

    function run(address circleMessageTransmitter) public returns (address deployedAddress) {
        vm.startBroadcast();
        (deployedAddress) = _deploy(circleMessageTransmitter);
        vm.stopBroadcast();
    }

    function _deploy(address circleMessageTransmitter) internal returns (address deployedAddress) {
        bytes32 salt = keccak256(abi.encodePacked(cctpWithExecutorVersion));
        CCTPv1ReceiveWithGasDropOff cctpWithExecutor =
            new CCTPv1ReceiveWithGasDropOff{salt: salt}(circleMessageTransmitter);

        return (address(cctpWithExecutor));
    }
}

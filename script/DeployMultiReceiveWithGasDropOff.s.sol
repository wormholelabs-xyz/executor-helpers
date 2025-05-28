// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {MultiReceiveWithGasDropOff, multiReceiverWithGasDropOffVersion} from "../src/MultiReceiveWithGasDropOff.sol";
import "forge-std/Script.sol";

// DeployMultiReceiveWithGasDropOff is a forge script to deploy the MultiReceiveWithGasDropOff contract. Use ./sh/deployMultiReceiveWithGasDropOff.sh to invoke this.
//    EVM_CHAIN_ID= MNEMONIC= ./sh/deployMultiReceiveWithGasDropOff.sh
contract DeployMultiReceiveWithGasDropOff is Script {
    function test() public {} // Exclude this from coverage report.

    function dryRun() public {
        _deploy();
    }

    function run() public returns (address deployedAddress) {
        vm.startBroadcast();
        (deployedAddress) = _deploy();
        vm.stopBroadcast();
    }

    function _deploy() internal returns (address deployedAddress) {
        bytes32 salt = keccak256(abi.encodePacked(multiReceiverWithGasDropOffVersion));
        MultiReceiveWithGasDropOff multiWithExecutor = new MultiReceiveWithGasDropOff{salt: salt}();

        return (address(multiWithExecutor));
    }
}

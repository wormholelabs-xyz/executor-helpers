// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {VAAv1ReceiveWithGasDropOff, vaaV1ReceiveWithGasDropOffVersion} from "../src/VAAv1ReceiveWithGasDropOff.sol";
import "forge-std/Script.sol";

// DeployVAAv1ReceiveWithGasDropOff is a forge script to deploy the VAAv1ReceiveWithGasDropOff contract. Use ./sh/deployVAAv1ReceiveWithGasDropOff.sh to invoke this.
//    EVM_CHAIN_ID= MNEMONIC= ./sh/deployVAAv1ReceiveWithGasDropOff.sh
contract DeployVAAv1ReceiveWithGasDropOff is Script {
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
        bytes32 salt = keccak256(abi.encodePacked(vaaV1ReceiveWithGasDropOffVersion));
        VAAv1ReceiveWithGasDropOff multiWithExecutor = new VAAv1ReceiveWithGasDropOff{salt: salt}();

        return (address(multiWithExecutor));
    }
}

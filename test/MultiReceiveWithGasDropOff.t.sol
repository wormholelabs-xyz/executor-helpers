// SPDX-License-Identifier: Apache 2
pragma solidity >=0.8.8 <0.9.0;

import "forge-std/Test.sol";

import "../src/MultiReceiveWithGasDropOff.sol";
import "../src/interfaces/IMultiReceiveWithGasDropOff.sol";

contract MockReceiver {
    bytes public message;

    function receiveMessage(bytes memory encodedMessage) external {
        message = encodedMessage;
    }
}

contract TestMultiReceiveWithGasDropOff is Test {
    MultiReceiveWithGasDropOff _multiReceiveWithDropOff;

    address _executor = address(0x123);
    address _payee = address(0x456);

    function setUp() public {
        string memory url = "https://ethereum-sepolia-rpc.publicnode.com";
        vm.createSelectFork(url);

        _multiReceiveWithDropOff = new MultiReceiveWithGasDropOff();

        // Give everyone some money to play with.
        vm.deal(_executor, 1 ether);
        vm.deal(_payee, 1 ether);
    }

    function test_invalidArgs() public {
        vm.startPrank(_executor);

        address[] memory contracts = new address[](2);
        bytes[] memory messages = new bytes[](5);

        uint256 dropOffValue = 100;

        vm.expectRevert(
            abi.encodeWithSelector(
                IMultiReceiveWithGasDropOff.InvalidParameters.selector, contracts.length, messages.length
            )
        );
        _multiReceiveWithDropOff.receiveMessages{value: dropOffValue}(contracts, messages, _payee);
    }

    function test_success() public {
        vm.startPrank(_executor);

        address[] memory contracts = new address[](5);
        bytes[] memory messages = new bytes[](5);

        for (uint256 idx = 0; idx < 5;) {
            contracts[idx] = address(new MockReceiver());
            messages[idx] = abi.encode("Hi, Mom!", idx);
            unchecked {
                idx++;
            }
        }

        uint256 startingBalance = address(_payee).balance;
        uint256 dropOffValue = 100;

        _multiReceiveWithDropOff.receiveMessages{value: dropOffValue}(contracts, messages, _payee);

        // Make sure the contracts received the right messages.
        for (uint256 idx = 0; idx < 5;) {
            assertEq(keccak256(MockReceiver(contracts[idx]).message()), keccak256(messages[idx]));
            unchecked {
                idx++;
            }
        }

        // Make sure the gas got dropped off.
        uint256 endingBalance = address(_payee).balance;
        assertEq(endingBalance, startingBalance + dropOffValue);
    }

    function test_successNoDropOff() public {
        vm.startPrank(_executor);

        address[] memory contracts = new address[](5);
        bytes[] memory messages = new bytes[](5);

        for (uint256 idx = 0; idx < 5;) {
            contracts[idx] = address(new MockReceiver());
            messages[idx] = abi.encode("Hi, Mom!", idx);
            unchecked {
                idx++;
            }
        }

        uint256 startingBalance = address(_payee).balance;
        uint256 dropOffValue = 0;

        _multiReceiveWithDropOff.receiveMessages{value: dropOffValue}(contracts, messages, _payee);

        // Make sure the contracts received the right messages.
        for (uint256 idx = 0; idx < 5;) {
            assertEq(keccak256(MockReceiver(contracts[idx]).message()), keccak256(messages[idx]));
            unchecked {
                idx++;
            }
        }

        // Make sure nothing got dropped off.
        uint256 endingBalance = address(_payee).balance;
        assertEq(endingBalance, startingBalance);
    }
}

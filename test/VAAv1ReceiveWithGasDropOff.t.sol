// SPDX-License-Identifier: Apache 2
pragma solidity >=0.8.8 <0.9.0;

import "forge-std/Test.sol";

import "../src/VAAv1ReceiveWithGasDropOff.sol";
import "../src/interfaces/IVAAv1ReceiveWithGasDropOff.sol";

contract MockReceiver {
    bytes public message;
    uint256 public paid;

    function executeVAAv1(bytes memory encodedMessage) external payable {
        paid = msg.value;
        message = encodedMessage;
    }
}

contract TestVAAv1ReceiveWithGasDropOff is Test {
    VAAv1ReceiveWithGasDropOff _vaaV1ReceiveWithDropOff;
    MockReceiver _mockReceiver;

    address _executor = address(0x123);
    address _payee = address(0x456);
    address _receiver;

    function setUp() public {
        string memory url = "https://ethereum-sepolia-rpc.publicnode.com";
        vm.createSelectFork(url);

        _vaaV1ReceiveWithDropOff = new VAAv1ReceiveWithGasDropOff();

        _mockReceiver = new MockReceiver();
        _receiver = address(_mockReceiver);

        // Give everyone some money to play with.
        vm.deal(_executor, 1 ether);
        vm.deal(_payee, 1 ether);
    }

    function test_invalidArgs() public {
        vm.startPrank(_executor);

        bytes memory message = "Hi, Mom!";

        vm.expectRevert(abi.encodeWithSelector(IVAAv1ReceiveWithGasDropOff.InvalidMsgValue.selector, 42, 100));
        _vaaV1ReceiveWithDropOff.receiveMessage{value: 42}(_receiver, message, _payee, 100);
    }

    function test_success() public {
        vm.startPrank(_executor);

        bytes memory message = "Hi, Mom!";

        uint256 startingBalance = address(_payee).balance;
        uint256 fee = 42;
        uint256 dropOffValue = 100;

        _vaaV1ReceiveWithDropOff.receiveMessage{value: fee + dropOffValue}(_receiver, message, _payee, dropOffValue);

        // Make sure the contract received the right message.
        assertEq(keccak256(_mockReceiver.message()), keccak256(message));

        // Make sure the receiver got paid the correct amount.
        assertEq(_mockReceiver.paid(), fee);

        // Make sure the gas got dropped off.
        uint256 endingBalance = address(_payee).balance;
        assertEq(endingBalance, startingBalance + dropOffValue);
    }

    function test_successNoDropOff() public {
        vm.startPrank(_executor);

        bytes memory message = "Hi, Mom!";

        uint256 startingBalance = address(_payee).balance;
        uint256 fee = 42;
        uint256 dropOffValue = 0;

        _vaaV1ReceiveWithDropOff.receiveMessage{value: fee + dropOffValue}(_receiver, message, _payee, dropOffValue);

        // Make sure the contract received the right message.
        assertEq(keccak256(_mockReceiver.message()), keccak256(message));

        // Make sure the receiver got paid the correct amount.
        assertEq(_mockReceiver.paid(), fee);

        // Make sure nothing got dropped off.
        uint256 endingBalance = address(_payee).balance;
        assertEq(endingBalance, startingBalance);
    }

    function test_successPayContractNothing() public {
        vm.startPrank(_executor);

        bytes memory message = "Hi, Mom!";

        uint256 startingBalance = address(_payee).balance;
        uint256 fee = 0;
        uint256 dropOffValue = 100;

        _vaaV1ReceiveWithDropOff.receiveMessage{value: fee + dropOffValue}(_receiver, message, _payee, dropOffValue);

        // Make sure the contract received the right message.
        assertEq(keccak256(_mockReceiver.message()), keccak256(message));

        // Make sure the receiver didn't get paid anything.
        assertEq(_mockReceiver.paid(), fee);

        // Make sure the gas got dropped off.
        uint256 endingBalance = address(_payee).balance;
        assertEq(endingBalance, startingBalance + dropOffValue);
    }
}

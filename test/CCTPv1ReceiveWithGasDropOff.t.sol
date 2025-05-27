// SPDX-License-Identifier: Apache 2
pragma solidity >=0.8.8 <0.9.0;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import "openzeppelin-contracts/contracts/proxy/ERC1967/ERC1967Upgrade.sol";

import "../src/CCTPv1ReceiveWithGasDropOff.sol";
import "../src/interfaces/ICCTPv1ReceiveWithGasDropOff.sol";
import "../src/interfaces/circle/ICircleV1MessageTransmitter.sol";

contract MockMessageTransmitter is ICircleV1MessageTransmitter {
    bool public immutable retVal;

    constructor() {
        retVal = true;
    }

    function receiveMessage(bytes calldata, bytes calldata) external view returns (bool success) {
        return retVal;
    }
}

contract MockCCTPv1ReceiveWithGasDropOff is CCTPv1ReceiveWithGasDropOff {
    constructor(address _circleMessageTransmitter) CCTPv1ReceiveWithGasDropOff(_circleMessageTransmitter) {}
}

contract TestCCTPv1ReceiveWithGasDropOff is Test {
    MockMessageTransmitter _circleMessageTransmitter;
    CCTPv1ReceiveWithGasDropOff _cctpWithExecutor;

    address _executor = address(0x123);
    address _payee = address(0x456);

    function setUp() public {
        _circleMessageTransmitter = new MockMessageTransmitter();
        _cctpWithExecutor = new CCTPv1ReceiveWithGasDropOff(address(_circleMessageTransmitter));

        string memory url = "https://ethereum-sepolia-rpc.publicnode.com";
        vm.createSelectFork(url);

        // Give everyone some money to play with.
        vm.deal(_executor, 1 ether);
        vm.deal(_payee, 1 ether);
    }

    function test_receiveMessage() public {
        vm.startPrank(_executor);

        uint256 startingBalance = address(_payee).balance;
        bytes memory message = "Hello, World!";
        bytes memory attestation = "Hi, Mom!";

        uint256 dropOffValue = 100;

        _cctpWithExecutor.receiveMessage{value: dropOffValue}(message, attestation, _payee);

        uint256 endingBalance = address(_payee).balance;
        assertEq(endingBalance, startingBalance + dropOffValue);
    }
}

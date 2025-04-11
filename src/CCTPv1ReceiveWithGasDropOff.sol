// SPDX-License-Identifier: Apache 2
pragma solidity ^0.8.19;

import "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

import {ICircleV1MessageTransmitter} from "./interfaces/circle/ICircleV1MessageTransmitter.sol";

import "./interfaces/ICCTPv1ReceiveWithGasDropOff.sol";

string constant cctpWithExecutorVersion = "CCTPv1ReceiveWithGasDropOff-0.0.1";

/// @title CCTPv1ReceiveWithGasDropOff
/// @author Executor Project Contributors.
/// @notice The CCTPv1ReceiveWithGasDropOff contract is a shim contract that does a Circle receive message and drops off gas at the specified address.
contract CCTPv1ReceiveWithGasDropOff is ICCTPv1ReceiveWithGasDropOff {
    ICircleV1MessageTransmitter public immutable circleMessageTransmitter;

    string public constant VERSION = cctpWithExecutorVersion;

    constructor(address _circleMessageTransmitter) {
        assert(_circleMessageTransmitter != address(0));
        circleMessageTransmitter = ICircleV1MessageTransmitter(_circleMessageTransmitter);
    }

    // ==================== External Interface ===============================================

    /// @inheritdoc ICCTPv1ReceiveWithGasDropOff
    function receiveMessage(bytes calldata message, bytes calldata attestation, address payeeAddress)
        external
        payable
    {
        // Do the receive.
        bool success = circleMessageTransmitter.receiveMessage(message, attestation);
        if (!success) {
            revert ReceiveMessageFailed();
        }

        if (msg.value > 0) {
            (bool dropOffSuccessful,) = payable(payeeAddress).call{value: msg.value}("");
            if (!dropOffSuccessful) {
                revert DropOffFailed(payeeAddress, msg.value);
            }
        }
    }
}

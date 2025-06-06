// SPDX-License-Identifier: Apache 2
pragma solidity ^0.8.19;

import "./interfaces/IVAAv1ReceiveWithGasDropOff.sol";
import "example-messaging-executor/evm/src/interfaces/IVaaV1Receiver.sol";

string constant vaaV1ReceiveWithGasDropOffVersion = "VAAv1ReceiveWithGasDropOff-0.0.1";

/// @title VAAv1ReceiveWithGasDropOff
/// @author Executor Project Contributors.
/// @notice The VAAv1ReceiveWithGasDropOff contract is a shim contract that receives a V1 VAA and drops off gas at the specified address.
contract VAAv1ReceiveWithGasDropOff is IVAAv1ReceiveWithGasDropOff {
    string public constant VERSION = vaaV1ReceiveWithGasDropOffVersion;

    // ==================== External Interface ===============================================

    /// @inheritdoc IVAAv1ReceiveWithGasDropOff
    function receiveMessage(address contractAddr, bytes calldata message, address payeeAddress, uint256 dropOffValue)
        external
        payable
    {
        if (msg.value < dropOffValue) {
            revert InvalidMsgValue(msg.value, dropOffValue);
        }

        uint256 value = msg.value - dropOffValue;
        IVaaV1Receiver(contractAddr).executeVAAv1{value: value}(message);

        if (dropOffValue > 0) {
            (bool dropOffSuccessful,) = payable(payeeAddress).call{value: dropOffValue}("");
            if (!dropOffSuccessful) {
                revert DropOffFailed(payeeAddress, dropOffValue);
            }
        }
    }
}

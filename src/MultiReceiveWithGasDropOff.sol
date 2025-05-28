// SPDX-License-Identifier: Apache 2
pragma solidity ^0.8.19;

import "./interfaces/IMultiReceiveWithGasDropOff.sol";
import "./interfaces/IReceiveMessage.sol";

string constant multiReceiverWithGasDropOffVersion = "MultiReceiveWithGasDropOff-0.0.1";

/// @title MultiReceiveWithGasDropOff
/// @author Executor Project Contributors.
/// @notice The MultiReceiveWithGasDropOff contract is a shim contract that does multiple message receives in a single call and drops off gas at the specified address.
contract MultiReceiveWithGasDropOff is IMultiReceiveWithGasDropOff {
    string public constant VERSION = multiReceiverWithGasDropOffVersion;

    // ==================== External Interface ===============================================

    /// @inheritdoc IMultiReceiveWithGasDropOff
    function receiveMessages(address[] calldata contracts, bytes[] calldata messages, address payeeAddress)
        external
        payable
    {
        if (contracts.length != messages.length) {
            revert InvalidParameters(contracts.length, messages.length);
        }

        uint256 len = contracts.length;
        for (uint256 idx = 0; idx < len;) {
            IReceiveMessage(contracts[idx]).receiveMessage(messages[idx]);
            unchecked {
                idx++;
            }
        }

        if (msg.value > 0) {
            (bool dropOffSuccessful,) = payable(payeeAddress).call{value: msg.value}("");
            if (!dropOffSuccessful) {
                revert DropOffFailed(payeeAddress, msg.value);
            }
        }
    }
}

// SPDX-License-Identifier: Apache 2
pragma solidity ^0.8.19;

interface IReceiveMessage {
    /// @notice Receive an attested message.
    /// @param encodedMessage The attested message.
    function receiveMessage(bytes memory encodedMessage) external;
}

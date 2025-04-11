// SPDX-License-Identifier: Apache 2
pragma solidity ^0.8.19;

struct ExecutorArgs {
    // The refund address used by the Executor.
    address refundAddress;
    // The signed quote to be passed into the Executor.
    bytes signedQuote;
    // The relay instructions to be passed into the Executor.
    bytes instructions;
}

interface ICCTPv1ReceiveWithGasDropOff {
    error ReceiveMessageFailed();
    error DropOffFailed(address, uint256);

    /**
     * @notice Receive a CCTP message and drops gas off at the specified address.
     * @param message CCTP Message bytes
     * @param attestation CCTP Attestation bytes
     * @param  payeeAddress address to receive the gas drop off
     */
    function receiveMessage(bytes calldata message, bytes calldata attestation, address payeeAddress)
        external
        payable;
}

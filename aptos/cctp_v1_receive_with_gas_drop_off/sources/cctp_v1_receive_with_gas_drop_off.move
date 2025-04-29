// SPDX-License-Identifier: Apache-2.0

// modified from https://github.com/circlefin/aptos-cctp/blob/4d8901d11e904d6d8b23dcab3ae513576cc4885d/packages/token_messenger_minter/scripts/handle_receive_message.move
module cctp_v1_receive_with_gas_drop_off::cctp_v1_receive_with_gas_drop_off {
    use aptos_framework::aptos_account;
    use message_transmitter::message_transmitter;
    use token_messenger_minter::token_messenger;

    public entry fun handle_receive_message_entry(
        caller: &signer,
        message: vector<u8>,
        attestation: vector<u8>,
        to: address,
        amount: u64
    ) {
        let receipt = message_transmitter::receive_message(
            caller,
            &message,
            &attestation
        );
        token_messenger::handle_receive_message(receipt);
        if (amount > 0) {
            aptos_account::transfer(caller, to, amount)
        }
    }
}

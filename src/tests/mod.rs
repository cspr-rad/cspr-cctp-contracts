#[cfg(test)]
mod test_setup {
    use crate::message_transmitter::{MessageTransmitterHostRef, MessageTransmitterInitArgs};
    use crate::stablecoin::StablecoinHostRef;
    use crate::stablecoin::StablecoinInitArgs;
    use crate::token_messenger_minter::{
        TokenMessengerMinterHostRef, TokenMessengerMinterInitArgs,
    };
    use crate::{generic_address, generic_address_to_contract_address};
    use crate::{
        message_transmitter::message::Message, token_messenger_minter::burn_message::BurnMessage,
    };
    use odra::casper_types::bytesrepr::Bytes;
    use odra::casper_types::U256;
    use odra::host::Deployer;
    use odra::host::HostEnv;
    use odra::{Address, Addressable};

    fn setup_cctp_contracts() -> (
        HostEnv,
        StablecoinHostRef,
        MessageTransmitterHostRef,
        TokenMessengerMinterHostRef,
        Address,
        Address,
        Address,
        Address,
    ) {
        let env = odra_test::env();
        let owner = env.get_account(0);
        let master_minter = env.get_account(1);
        let blacklister = env.get_account(2);
        let controller = env.get_account(3);
        let stablecoin_init_args = StablecoinInitArgs {
            symbol: "USDC".to_string(),
            name: "USDCoin".to_string(),
            decimals: 6,
            initial_supply: 1_000_000_000.into(),
            master_minter_list: vec![master_minter],
            pauser_list: vec![],
            blacklister,
            modality: Some(crate::stablecoin::utils::StablecoinModality::MintAndBurn),
        };
        let stablecoin: StablecoinHostRef = StablecoinHostRef::deploy(&env, stablecoin_init_args);

        let message_transmitter_init_args = MessageTransmitterInitArgs {
            local_domain: 31u32,
            version: 2u32,
            max_message_body_size: 1_000_000.into(),
            next_available_nonce: 0u64,
            signature_threshold: 0u32,
            owner,
        };
        let message_transmitter: MessageTransmitterHostRef =
            MessageTransmitterHostRef::deploy(&env, message_transmitter_init_args);

        let token_messenger_minter_init_args = TokenMessengerMinterInitArgs {
            version: 2u32,
            local_message_transmitter: *message_transmitter.address(),
            max_burn_amount_per_message: U256::from(100),
            owner,
        };
        let token_messenger_minter: TokenMessengerMinterHostRef =
            TokenMessengerMinterHostRef::deploy(&env, token_messenger_minter_init_args);

        (
            env,
            stablecoin,
            message_transmitter,
            token_messenger_minter,
            owner,
            master_minter,
            blacklister,
            controller,
        )
    }
    #[test]
    fn test_deposit_for_burn() {
        let (
            env,
            mut stablecoin,
            message_transmitter,
            mut token_messenger_minter,
            owner,
            master_minter,
            ..,
            controller,
        ) = setup_cctp_contracts();
        let fake_minter = env.get_account(4);
        let user = env.get_account(5);
        env.set_caller(master_minter);
        stablecoin.configure_controller(&controller, &fake_minter.address());
        env.set_caller(controller);
        stablecoin.configure_minter_allowance(100.into());
        env.set_caller(fake_minter);
        // use fake minter to mint 10 tokens - we want to test depositForBurn, not receive message
        stablecoin.mint(&user, 10.into());
        env.set_caller(master_minter);
        stablecoin.configure_controller(&controller, token_messenger_minter.address());
        env.set_caller(controller);
        stablecoin.configure_minter_allowance(100.into());
        env.set_caller(owner);
        token_messenger_minter.link_token_pair(*stablecoin.address(), [0u8; 32], 0u32);
        let mint_recipient: [u8; 32] = [1u8; 32];
        token_messenger_minter.add_remote_token_messenger(0u32, [2u8; 32]);
        env.set_caller(user);
        stablecoin.approve(token_messenger_minter.address(), &10.into());
        token_messenger_minter.deposit_for_burn(10, 0u32, mint_recipient, *stablecoin.address());
        assert!(
            env.emitted(token_messenger_minter.address(), "DepositForBurn"),
            "DepositForBurn event not emitted"
        );
        assert!(
            env.emitted(message_transmitter.address(), "MessageSent"),
            "MessageSent event not emitted"
        )
    }

    #[test]
    fn test_receive_message_from_remote_domain() {
        let (
            env,
            mut stablecoin,
            mut message_transmitter,
            mut token_messenger_minter,
            owner,
            master_minter,
            ..,
            controller,
        ) = setup_cctp_contracts();
        let remote_token_address: [u8; 32] = [10u8; 32];
        let remote_token_messenger: [u8; 32] = [11u8; 32];
        let remote_domain: u32 = 0;
        let mint_recipient: Address = env.get_account(0);
        env.set_caller(master_minter);
        stablecoin.configure_controller(&controller, token_messenger_minter.address());
        env.set_caller(controller);
        stablecoin.configure_minter_allowance(100.into());
        env.set_caller(owner);
        // message sender must be a remote_token_messenger
        token_messenger_minter.add_remote_token_messenger(remote_domain, remote_token_messenger);
        token_messenger_minter.link_token_pair(
            *stablecoin.address(),
            remote_token_address,
            remote_domain,
        );
        let message_body: Vec<u8> = BurnMessage::format_message(
            2,
            &remote_token_address,
            &generic_address(mint_recipient),
            10,
            &remote_token_messenger,
        );
        let message: Vec<u8> = Message::format_message(
            2,
            remote_domain,
            32,
            0,
            &remote_token_messenger,
            &generic_address(token_messenger_minter.address().clone()),
            &[0u8; 32],
            &message_body,
        );
        let message_typed: Message = Message::new(2, &message);
        let message_recipient = message_typed.recipient();
        let message_recipient_address = generic_address_to_contract_address(message_recipient);
        assert_eq!(&message_recipient_address, token_messenger_minter.address());
        message_transmitter.receive_message(Bytes::from(message), Bytes::from(vec![]));
        assert!(
            env.emitted(message_transmitter.address(), "MessageReceived"),
            "MessageReceived event not emitted"
        );
    }

    #[test]
    fn test_replace_message() {
        let (
            env,
            mut stablecoin,
            message_transmitter,
            mut token_messenger_minter,
            owner,
            master_minter,
            ..,
            controller,
        ) = setup_cctp_contracts();
        let remote_token_address: [u8; 32] = [10u8; 32];
        let remote_token_messenger: [u8; 32] = [11u8; 32];
        let remote_domain: u32 = 0;
        let mint_recipient: Address = env.get_account(0);
        env.set_caller(master_minter);
        stablecoin.configure_controller(&controller, token_messenger_minter.address());
        env.set_caller(controller);
        stablecoin.configure_minter_allowance(100.into());
        env.set_caller(owner);
        // message sender must be a remote_token_messenger
        token_messenger_minter.add_remote_token_messenger(remote_domain, remote_token_messenger);
        token_messenger_minter.link_token_pair(
            *stablecoin.address(),
            remote_token_address,
            remote_domain,
        );
        let message_body: Vec<u8> = BurnMessage::format_message(
            2,
            &remote_token_address,
            &generic_address(mint_recipient),
            10,
            &generic_address(owner),
        );
        let message: Vec<u8> = Message::format_message(
            2,
            31,
            0,
            0,
            &generic_address(token_messenger_minter.address().clone()),
            &generic_address(token_messenger_minter.address().clone()),
            &[0u8; 32],
            &message_body,
        );
        let message_typed: Message = Message::new(2, &message);
        let message_recipient = message_typed.recipient();
        let message_recipient_address = generic_address_to_contract_address(message_recipient);
        assert_eq!(&message_recipient_address, token_messenger_minter.address());
        token_messenger_minter.replace_deposit_for_burn(
            Bytes::from(message.clone()),
            Bytes::from(vec![]),
            [0u8; 32],
            [1u8; 32],
        );
        assert!(
            env.emitted(message_transmitter.address(), "MessageSent"),
            "MessageSent event not emitted"
        );
    }
}

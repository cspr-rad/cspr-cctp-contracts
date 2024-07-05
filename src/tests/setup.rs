#[cfg(test)]
mod test_setup{
    use odra::casper_types::U256;
    use odra::host::Deployer;
    use odra::host::HostEnv;
    use odra::Addressable;
    use crate::message_transmitter;
    use crate::stablecoin::StablecoinHostRef;
    use crate::stablecoin::StablecoinInitArgs;
    use crate::token_messenger_minter::{
        TokenMessengerMinterHostRef, TokenMessengerMinterInitArgs,
    };
    use crate::message_transmitter::{
        MessageTransmitterHostRef, MessageTransmitterInitArgs
    };
    #[test]
    fn setup_cctp_contracts_and_test_deposit_for_burn(){
        // install stablecoin
        // install message transmitter
        // install token messenger minter
        // call stablecoin to register controller pair (with masterminter)
        // set an allowance for the minter (with controller)
        // link a tokenpair to the token messenger minter (with owner)
        let env = odra_test::env();
        let owner = env.get_account(0);
        let master_minter = env.get_account(1);
        let blacklister = env.get_account(2);
        let controller = env.get_account(3);
        let fake_minter = env.get_account(4);
        let stablecoin_init_args = StablecoinInitArgs{
            symbol: "USDC".to_string(),
            name: "USDCoin".to_string(),
            decimals: 6,
            initial_supply: 1_000_000_000.into(),
            master_minter_list: vec![master_minter],
            pauser_list: vec![],
            blacklister,
            modality: Some(crate::stablecoin::utils::StablecoinModality::MintAndBurn)
        };
        let mut stablecoin: StablecoinHostRef = StablecoinHostRef::deploy(&env, stablecoin_init_args);

        let message_transmitter_init_args = MessageTransmitterInitArgs{
            local_domain: 31u32,
            version: 2u32,
            max_message_body_size: 1_000_000.into(),
            next_available_nonce: 0u64,
            signature_threshold: 1u32,
            owner
        };
        let mut message_transmitter: MessageTransmitterHostRef = MessageTransmitterHostRef::deploy(&env, message_transmitter_init_args);

        let token_messenger_minter_init_args = TokenMessengerMinterInitArgs{
            version: 2u32,
            local_message_transmitter: *message_transmitter.address(),
            owner
        };
        let mut token_messenger_minter: TokenMessengerMinterHostRef = TokenMessengerMinterHostRef::deploy(&env, token_messenger_minter_init_args);
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
        token_messenger_minter.link_token_pair(*stablecoin.address(), [0u8;32], 0u32);   
        let mint_recipient: [u8;32] = [1u8;32];
        env.set_caller(owner);
        token_messenger_minter.add_remote_token_messenger(0u32, [2u8;32]);
        env.set_caller(user);
        stablecoin.approve(token_messenger_minter.address(), &10.into());
        token_messenger_minter.deposit_for_burn(10, 0u32, mint_recipient, *stablecoin.address());
        assert!(
            env.emitted(token_messenger_minter.address(), "DepositForBurn"),
            "DepositForBurn event not emitted"
        );
    }
}
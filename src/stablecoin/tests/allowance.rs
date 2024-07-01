#[cfg(test)]
mod allowance_tests {
    use crate::stablecoin::errors::Error::InsufficientAllowance;
    use crate::stablecoin::cross_contract_test::StablecoinClientContractHostRef;
    use crate::stablecoin::setup_tests::{
        invert_address, setup, setup_with_args, ALLOWANCE_AMOUNT_1, ALLOWANCE_AMOUNT_2,
        TOKEN_DECIMALS, TOKEN_NAME, TOKEN_SYMBOL, TOKEN_TOTAL_SUPPLY, TRANSFER_AMOUNT_1,
    };
    use crate::stablecoin::{StablecoinHostRef, StablecoinInitArgs};
    use core::ops::Add;
    use odra::casper_types::U256;
    use odra::host::{Deployer, HostRef, NoArgs};
    use odra::Address;

    fn test_approve_for(
        stablecoin: &mut StablecoinHostRef,
        sender: Address,
        owner: Address,
        spender: Address,
    ) {
        let amount = TRANSFER_AMOUNT_1.into();
        assert_eq!(stablecoin.allowance(&owner, &spender), 0.into());
        stablecoin.env().set_caller(sender);
        stablecoin.approve(&spender, &amount);
        assert!(
            stablecoin.env().emitted(stablecoin, "SetAllowance"),
            "SetAllowance event not emitted"
        );
        assert_eq!(stablecoin.allowance(&owner, &spender), amount);
        stablecoin.approve(&spender, &(amount.add(U256::one())));
        assert!(
            stablecoin.env().emitted(stablecoin, "SetAllowance"),
            "SetAllowance event not emitted"
        );
        assert_eq!(
            stablecoin.allowance(&owner, &spender),
            amount.add(U256::one())
        );
        let inverted_owner = invert_address(owner);
        let inverted_spender = invert_address(spender);
        assert_eq!(
            stablecoin.allowance(&inverted_owner, &inverted_spender),
            U256::zero()
        );
    }

    #[test]
    fn should_approve_funds() {
        let (env, .., mut stablecoin) = setup();
        let owner = env.caller();
        let alice = env.get_account(1);
        let token_address = *stablecoin.address();
        let client_contract = StablecoinClientContractHostRef::deploy(&env, NoArgs);
        let another_client_contract =
            StablecoinClientContractHostRef::deploy(&env, NoArgs);
        let client_contract_address = client_contract.address();
        let another_client_contract_address = another_client_contract.address();
        test_approve_for(&mut stablecoin, owner, owner, alice);
        stablecoin.approve(client_contract_address, &ALLOWANCE_AMOUNT_1.into());
        assert_eq!(
            stablecoin.allowance(&owner, client_contract_address),
            ALLOWANCE_AMOUNT_1.into()
        );
        client_contract.transfer_from_as_stored_contract(
            token_address,
            owner,
            *client_contract_address,
            ALLOWANCE_AMOUNT_1.into(),
        );
        assert_eq!(
            stablecoin.balance_of(client_contract_address),
            ALLOWANCE_AMOUNT_1.into()
        );
        client_contract.approve_as_stored_contract(
            token_address,
            *another_client_contract_address,
            ALLOWANCE_AMOUNT_1.into(),
        );
        assert_eq!(
            stablecoin.allowance(client_contract_address, another_client_contract_address),
            ALLOWANCE_AMOUNT_1.into()
        );
        another_client_contract.transfer_from_as_stored_contract(
            token_address,
            *client_contract_address,
            *another_client_contract_address,
            ALLOWANCE_AMOUNT_1.into(),
        );
        assert!(
            env.emitted(&stablecoin, "TransferFrom"),
            "TransferFrom event not emitted"
        );
        assert_eq!(
            stablecoin.balance_of(another_client_contract_address),
            ALLOWANCE_AMOUNT_1.into()
        );
    }

    #[test]
    fn should_not_transfer_from_without_enough_allowance() {
        let (env, .., mut stablecoin) = setup();
        let owner = env.caller();
        let alice = env.get_account(1);
        stablecoin.approve(&alice, &ALLOWANCE_AMOUNT_1.into());
        assert_eq!(
            stablecoin.allowance(&owner, &alice),
            ALLOWANCE_AMOUNT_1.into()
        );
        env.set_caller(alice);
        let result =
            stablecoin.try_transfer_from(&owner, &alice, &U256::from(ALLOWANCE_AMOUNT_1 + 1));
        assert_eq!(result.err().unwrap(), InsufficientAllowance.into());
        stablecoin.transfer_from(&owner, &alice, &U256::from(ALLOWANCE_AMOUNT_1));
    }

    #[test]
    fn test_decrease_allowance() {
        let (env, .., mut stablecoin) = setup();
        let owner = env.caller();
        let alice = env.get_account(1);
        stablecoin.approve(&alice, &ALLOWANCE_AMOUNT_1.into());
        assert_eq!(
            stablecoin.allowance(&owner, &alice),
            ALLOWANCE_AMOUNT_1.into()
        );
        stablecoin.decrease_allowance(&alice, &ALLOWANCE_AMOUNT_2.into());
        assert!(
            env.emitted(&stablecoin, "DecreaseAllowance"),
            "DecreaseAllowance event not emitted"
        );
        assert_eq!(
            stablecoin.allowance(&owner, &alice),
            (ALLOWANCE_AMOUNT_1 - ALLOWANCE_AMOUNT_2).into()
        );
        stablecoin.increase_allowance(&alice, &ALLOWANCE_AMOUNT_1.into());
        assert!(
            env.emitted(&stablecoin, "IncreaseAllowance"),
            "IncreaseAllowance event not emitted"
        );
        assert_eq!(
            stablecoin.allowance(&owner, &alice),
            ((ALLOWANCE_AMOUNT_1 * 2) - ALLOWANCE_AMOUNT_2).into()
        );
    }

    #[test]
    fn test_increase_minter_allowance() {
        let env = odra_test::env();
        let master_minter = env.get_account(1);
        let controller_1 = env.get_account(2);
        let minter_1 = env.get_account(3);
        let blacklister = env.get_account(4);
        let args = StablecoinInitArgs {
            symbol: TOKEN_SYMBOL.to_string(),
            name: TOKEN_NAME.to_string(),
            decimals: TOKEN_DECIMALS,
            initial_supply: TOKEN_TOTAL_SUPPLY.into(),
            master_minter_list: vec![master_minter],
            owner_list: vec![],
            pauser_list: vec![],
            blacklister: blacklister,
            modality: Some(crate::stablecoin::utils::StablecoinModality::MintAndBurn),
        };
        let mut stablecoin = setup_with_args(&env, args);
        env.set_caller(master_minter);
        stablecoin.configure_controller(&controller_1, &minter_1);
        env.set_caller(controller_1);
        stablecoin.increase_minter_allowance(U256::from(10));
        assert_eq!(stablecoin.minter_allowance(&minter_1), U256::from(10));
    }
    #[test]
    fn test_decrease_minter_allowance() {
        let env = odra_test::env();
        let master_minter = env.get_account(1);
        let controller_1 = env.get_account(2);
        let minter_1 = env.get_account(3);
        let blacklister = env.get_account(4);
        let args = StablecoinInitArgs {
            symbol: TOKEN_SYMBOL.to_string(),
            name: TOKEN_NAME.to_string(),
            decimals: TOKEN_DECIMALS,
            initial_supply: TOKEN_TOTAL_SUPPLY.into(),
            master_minter_list: vec![master_minter],
            owner_list: vec![],
            pauser_list: vec![],
            blacklister: blacklister,
            modality: Some(crate::stablecoin::utils::StablecoinModality::MintAndBurn),
        };
        let mut stablecoin = setup_with_args(&env, args);
        env.set_caller(master_minter);
        stablecoin.configure_controller(&controller_1, &minter_1);
        env.set_caller(controller_1);
        stablecoin.increase_minter_allowance(U256::from(10));
        stablecoin.decrease_minter_allowance(U256::from(5));
        assert_eq!(stablecoin.minter_allowance(&minter_1), U256::from(5));
    }
}

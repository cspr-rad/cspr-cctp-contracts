#[cfg(test)]
mod mint_and_burn_tests {
    use crate::stablecoin::setup_tests::setup;
    use odra::casper_types::U256;
    use odra::host::HostRef;

    #[test]
    fn test_stablecoin_mint() {
        let (env, master_minter, controller_1, minter_1, .., user, mut cep18_token) = setup();
        cep18_token.env().set_caller(master_minter);
        cep18_token.configure_controller(&controller_1, &minter_1);
        cep18_token.env().set_caller(controller_1);
        cep18_token.configure_minter_allowance(U256::from(10));
        cep18_token.env().set_caller(minter_1);
        cep18_token.mint(&user, U256::from(10));
        assert_eq!(cep18_token.minter_allowance(&minter_1), U256::from(0));
        assert!(env.emitted(&cep18_token, "Mint"), "Mint event not emitted")
    }

    #[test]
    fn test_stablecoin_burn() {
        let (env, master_minter, controller_1, minter_1, .., mut cep18_token) = setup();
        cep18_token.env().set_caller(master_minter);
        cep18_token.configure_controller(&controller_1, &minter_1);
        cep18_token.env().set_caller(controller_1);
        cep18_token.configure_minter_allowance(U256::from(10));
        cep18_token.env().set_caller(minter_1);
        cep18_token.mint(&minter_1, U256::from(10));
        assert!(env.emitted(&cep18_token, "Mint"), "Mint event not emitted");
        cep18_token.burn(U256::from(10));
        assert!(env.emitted(&cep18_token, "Burn"), "Burn event not emitted");
    }
}

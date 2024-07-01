#[cfg(test)]
mod stablecoin_tests{
    use odra::host::{Deployer, HostEnv};
    use crate::stablecoin::{StablecoinHostRef, StablecoinInitArgs};
    use crate::stablecoin;
    #[test]
    pub fn test_setup(){
        const TOKEN_NAME: &str = "USDCoin";
        const TOKEN_SYMBOL: &str = "USDC";
        const TOKEN_DECIMALS: u8 = 100;
        const TOKEN_TOTAL_SUPPLY: u64 = 1_000_000_000;
        let env = odra_test::env();
        let master_minter = env.get_account(1);
        let controller_1 = env.get_account(2);
        let minter_1 = env.get_account(3);
        let blacklister = env.get_account(4);
        let pauser = env.get_account(5);
        let user = env.get_account(6);
        let args = StablecoinInitArgs {
            symbol: TOKEN_SYMBOL.to_string(),
            name: TOKEN_NAME.to_string(),
            decimals: TOKEN_DECIMALS,
            initial_supply: TOKEN_TOTAL_SUPPLY.into(),
            master_minter_list: vec![master_minter],
            owner_list: vec![],
            pauser_list: vec![pauser],
            blacklister: blacklister,
            modality: Some(stablecoin::utils::StablecoinModality::MintAndBurn),
        };
        let stablecoin = setup_with_args(&env, args);
        (
            env,
            master_minter,
            controller_1,
            minter_1,
            blacklister,
            pauser,
            user,
            stablecoin,
        );
    }
    fn setup_with_args(env: &HostEnv, args: StablecoinInitArgs) -> StablecoinHostRef {
        StablecoinHostRef::deploy(env, args)
    }
}
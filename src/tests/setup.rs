#[cfg(test)]
mod test_setup{
    use odra::host::Deployer;
    use odra::host::HostEnv;
    use crate::token_messenger_minter::{
        TokenMessengerMinterHostRef, TokenMessengerMinterInitArgs,
    };
    use crate::message_transmitter::{
        MessageTransmitterHostRef, MessageTransmitterInitArgs
    };
    #[test]
    fn setup_cctp_contracts(){
        // install stablecoin
        // install message transmitter
        // install token messenger minter
        // call stablecoin to register controller pair (with masterminter)
        // set an allowance for the minter (with controller)
        // link a tokenpair to the token messenger minter (with owner)

    }
}
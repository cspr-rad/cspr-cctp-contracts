#[cfg(test)]
mod install_message_transmitter {
    use crate::message_transmitter::setup_tests::setup;

    #[test]
    fn test_install() {
        let (_env, _message_transmitter) = setup();
    }
}

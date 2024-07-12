/// Error enum for the MessageTransmitter contract.
#[odra::odra_error]
pub enum Error {
    InsufficientRights = 50000,
    ContractIsPaused = 50001,
    InvalidMessageRecipient = 50002,
    InvalidSignatureRecoveryId = 50003,
    InvalidAttestationLength = 50004,
}

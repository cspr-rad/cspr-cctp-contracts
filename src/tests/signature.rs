use alloy::primitives::Keccak256;
use k256::ecdsa::{RecoveryId, Signature, SigningKey, VerifyingKey};

use crate::EthAddress;

pub fn construct_keypair(bytes: [u8;32]) -> (SigningKey, VerifyingKey){
    let sk: SigningKey = SigningKey::from_slice(&bytes).unwrap();
    let vk: VerifyingKey = sk.verifying_key().clone(); 
    (sk, vk)
}


pub fn recover_ethereum_address(pubkey: &[u8; 64]) -> EthAddress {
    let mut hasher = Keccak256::new();
    hasher.update(pubkey);
    let hash = hasher.finalize();
    hash.as_slice()[12..]
        .try_into()
        .expect("Failed to fit pubkey into slice")
}

pub fn sign_message(sk: SigningKey, message_hash: &[u8;32]) -> [u8;65]{
    let (signature, recovery_id): (Signature, RecoveryId) = sk
    .sign_prehash_recoverable(message_hash)
    .unwrap();
    let mut signature_bytes = signature.to_bytes().to_vec();
    signature_bytes.push(recovery_id.to_byte() + 27u8);
    signature_bytes.try_into().unwrap()
}
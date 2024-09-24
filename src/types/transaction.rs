use serde::{Serialize,Deserialize};
use crate::types::hash::{H256, Hashable};
use ring::signature::{Ed25519KeyPair, KeyPair, Signature};
use rand::{thread_rng, Rng};
use crate::types::address::Address;

use super::key_pair;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    pub sender: Address,
    pub receiver: Address,
    pub value: i64,
    pub nonce: u32,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl Hashable for SignedTransaction {
    fn hash(&self) -> H256 {
        let data = bincode::serialize(self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &data).into()
    }
}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    let data = bincode::serialize(t).unwrap();
    key.sign(&data)
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &[u8], signature: &[u8]) -> bool {
    let data = bincode::serialize(t).unwrap();
    let peer_public_key = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key);
    peer_public_key.verify(&data, signature).is_ok()
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_transaction() -> Transaction {
    let mut rng = rand::thread_rng();
    Transaction {
        sender: Address::from(rng.gen::<[u8; 20]>()),
        receiver: Address::from(rng.gen::<[u8; 20]>()),
        value: rng.gen(),
        nonce: rng.gen(),
    }
}

// #[cfg(any(test, test_utilities))]
pub fn generate_random_signed_transaction() -> SignedTransaction {
    let mut rng = thread_rng();

    let keys = key_pair::random();
    let public_key: Vec<u8> = keys.public_key().as_ref().to_vec();

    let transaction = Transaction {
        sender: Address::from_public_key_bytes(&public_key),
        receiver: Address::from(rng.gen::<[u8; 20]>()),
        value: rng.gen::<i64>(),
        nonce: rng.gen::<u32>(),
    };

    let signature: Vec<u8> = sign(&transaction, &keys).as_ref().to_vec();

    SignedTransaction {
        transaction,
        signature,
        public_key,
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::key_pair;
    use ring::signature::KeyPair;


    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, key.public_key().as_ref(), signature.as_ref()));
    }
    #[test]
    fn sign_verify_two() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        let key_2 = key_pair::random();
        let t_2 = generate_random_transaction();
        assert!(!verify(&t_2, key.public_key().as_ref(), signature.as_ref()));
        assert!(!verify(&t, key_2.public_key().as_ref(), signature.as_ref()));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
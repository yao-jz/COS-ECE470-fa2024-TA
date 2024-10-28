use std::collections::HashMap;
use std::hash::Hash;
use ring::signature::{Ed25519KeyPair, KeyPair, Signature};
use crate::types::key_pair;
use crate::types::address::Address;
use crate::types::transaction::{SignedTransaction, verify};

#[derive(Debug, Clone, Default)]
pub struct State {
    pub data: HashMap<Address, Account>,
    pub my_account: HashMap<Address, [u8; 32]>,
}

impl State {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            my_account: HashMap::new(),
        }
    }

    pub fn ico() -> Self {
        let mut state = Self::new();
        let seed = [0; 32];
        let u8seed: &[u8] = &seed;
        let keys = Ed25519KeyPair::from_seed_unchecked(u8seed).unwrap();
        let public_key: Vec<u8> = keys.public_key().as_ref().to_vec();
        let addr =Address::from_public_key_bytes(&public_key);
        println!("ICO address: {:?}", addr);
        
        state.my_account.insert(addr.clone(), seed);
        state.insert(&addr);
        state.update(&addr, i64::MAX);
        println!("ICO Account: {:?}", state.get_account(&addr));
        state
    }

    pub fn get_account(&self, address: &Address) -> Option<&Account> {
        self.data.get(address)
    }

    pub fn get_account_mut(&mut self, address: &Address) -> Option<&mut Account> {
        self.data.get_mut(address)
    }

    pub fn insert(&mut self, address: &Address) -> &mut Account {
        let account = Account::new(address);
        self.data.insert(address.clone(), account.clone());
        self.data.get_mut(address).unwrap()
    }

    pub fn process(&mut self, signed_transaction: &SignedTransaction) {
        let amount = signed_transaction.transaction.value;
        let sender = &signed_transaction.transaction.sender;
        let receiver = &signed_transaction.transaction.receiver;

        self.update(sender, -amount);
        self.update(receiver, amount);
    }

    pub fn update(&mut self, address: &Address, amount: i64) {
        match self.data.get_mut(address) {
            Some(balance) => {
                balance.update(amount);
            },
            None => {
                let mut account = self.insert(address);
                account.update(amount);
            }
        }
    }

    pub fn remove(&mut self, address: &Address, amount: u32) {
        self.data.remove(address);
    }

    pub fn is_transaction_valid(&self, signed_transaction: &SignedTransaction) -> bool {
        let address = &signed_transaction.transaction.sender;
        let amount = signed_transaction.transaction.value;
        let nonce = signed_transaction.transaction.nonce;
        let public_key = &signed_transaction.public_key;
        let signature = &signed_transaction.signature;

        verify(&signed_transaction.transaction, public_key, signature)
        && Self::check_public_key(address, public_key) 
        && self.check_balance(address, amount) 
        && self.check_nonce(address, nonce)
    }

    pub fn check_public_key(address: &Address, public_key: &Vec<u8>) -> bool {
        if address == &Address::from([0; 20]) {
            return true;
        }
        address == &Address::from_public_key_bytes(public_key)
    }

    pub fn check_balance(&self, address: &Address, amount: i64) -> bool {
        match self.data.get(address) {
            Some(balance) => {
                balance.has_balance(amount)
            },
            None => false
        }
    }

    pub fn check_nonce(&self, address: &Address, suggested_nonce: u32) -> bool {
        match self.data.get(address) {
            Some(balance) => {
                suggested_nonce == (balance.nonce + 1)
            },
            None => false
        }
    }

    pub fn to_string(&self) -> Vec<String> {
        let v = self.data.clone();
        let mut v_string: Vec<String> = v.into_values().map(|h|h.to_string()).collect();
        v_string.sort();
        v_string
    }
}


#[derive(Debug, Clone, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Account {
    pub nonce: u32,
    pub balance: i64,
    pub address: Address,
}

impl Account {
    pub fn new(address: &Address) -> Self {
        Self {
            nonce: 0,
            balance: 0,
            address: address.clone(),
        }
    }

    pub fn update(&mut self, amount: i64) {
        println!("Updating account: {:?}", self);
        println!("Amount: {:?}", amount);
        println!("Nonce: {:?}", self.nonce);
        println!("Balance: {:?}", self.balance);
        self.balance += amount;
        self.nonce += 1;
    }
    
    fn has_balance(&self, amount: i64) -> bool {
        self.balance >= amount
    }
}

impl std::fmt::Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Account {{ nonce: {}, balance: {}, address: {} }}", self.nonce, self.balance, self.address)
    }
}
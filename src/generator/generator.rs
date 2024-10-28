use log::info;
use indexmap::IndexMap;
use crate::types::state::Account;
use crate::types::transaction::sign;
use rand::Rng;
use crate::types::address::Address;
use crate::types::hash::Hashable;
use crate::types::transaction::Transaction;
use std::time;
use std::thread;
use crate::network::message::Message;
use crate::network::server::Handle as ServerHandle;
use crate::types::transaction::generate_random_signed_transaction;
use crate::types::state::State;
use crate::blockchain::Blockchain;
use crate::types::key_pair;
use ring::signature::{Ed25519KeyPair, KeyPair, Signature};
use crate::types::transaction::SignedTransaction;
use crate::types::mempool::Mempool;
use std::sync::{Arc, Mutex};
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};

#[derive(Clone)]
pub struct TransactionGenerator {
    server: ServerHandle,
    mempool: Arc<Mutex<Mempool>>,
    blockchain: Arc<Mutex<Blockchain>>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct GeneratorAccount {

}

impl TransactionGenerator {
    pub fn new(
        server: &ServerHandle,
        mempool: &Arc<Mutex<Mempool>>,
        blockchain: &Arc<Mutex<Blockchain>>,
    ) -> Self {
        Self {
            server: server.clone(),
            mempool: Arc::clone(mempool),
            blockchain: Arc::clone(blockchain),
        }
    }

    pub fn start(self, theta: u64) {
        thread::Builder::new()
            .name("transaction-generator".to_string())
            .spawn(move || {
                self.generate_transactions(theta);
            })
            .unwrap();
        println!("Transaction generator started");
    }

    fn generate_transactions(&self, theta: u64) {
        let mut rng = rand::thread_rng();
        loop {
            {
                let mut blockchain = self.blockchain.lock().unwrap();
                let tip = blockchain.tip;
                let state: &mut State = blockchain.states.get_mut(&tip).unwrap();
                let address_list = state.my_account.keys().cloned().collect::<Vec<_>>();
                let receiver_list = state.data.keys().cloned().collect::<Vec<_>>();

                let mut sender = address_list[rng.gen_range(0..address_list.len())];
                while None == state.get_account(&sender) || state.get_account(&sender).unwrap().balance <= 1 {
                    sender = address_list[rng.gen_range(0..address_list.len())];
                    if None != state.get_account(&sender) {
                        let sender_balance = state.get_account(&sender).unwrap().balance;
                        if sender_balance > 1 {
                            break;
                        }
                    }
                }
                let sender_seed: [u8; 32] = state.my_account.get(&sender).unwrap().clone();
                let sender_keys = Ed25519KeyPair::from_seed_unchecked(&sender_seed[..]).unwrap();
                let sender_public_key: Vec<u8> = sender_keys.public_key().as_ref().to_vec();
                // 20% possiblity receiver is among the existing addresses
                // 80% possibility receiver is a new random address (need to generate new key pair)
                let old_address = rng.gen_range(0..10) < 2;
                let receiver = if old_address {
                    let r = receiver_list[rng.gen_range(0..receiver_list.len())];
                    if r == sender {
                        // if receiver is the same as sender, generate a new random address
                        let seed: [u8; 32] = rng.gen();
                        let u8seed: &[u8] = &seed;
                        let keys = Ed25519KeyPair::from_seed_unchecked(u8seed).unwrap();
                        let public_key: Vec<u8> = keys.public_key().as_ref().to_vec();
                        let receiver_address = Address::from_public_key_bytes(&public_key);
                        state.my_account.insert(receiver_address.clone(), seed);
                        // state.data.insert(receiver_address.clone(), Account::new(&receiver_address));
                        receiver_address
                    } else {
                        r
                    }
                } else {
                    let seed: [u8; 32] = rng.gen();
                    let u8seed: &[u8] = &seed;
                    let keys = Ed25519KeyPair::from_seed_unchecked(u8seed).unwrap();
                    let public_key: Vec<u8> = keys.public_key().as_ref().to_vec();
                    let receiver_address = Address::from_public_key_bytes(&public_key);
                    state.my_account.insert(receiver_address.clone(), seed);
                    // state.data.insert(receiver_address.clone(), Account::new(&receiver_address));
                    receiver_address
                };
                let sender_balance = state.get_account(&sender).unwrap().balance;
                let min_value = if sender_balance > 1000000 { 1000000 } else { sender_balance };
                let value = rng.gen_range(1..min_value);
                let nonce = state.get_account(&sender).unwrap().nonce;
                let transaction = Transaction {
                    sender,
                    receiver,
                    value,
                    nonce: nonce + 1,
                };
                let signature: Vec<u8> = sign(&transaction, &sender_keys).as_ref().to_vec();
                let signed_transaction = SignedTransaction {
                    transaction,
                    signature,
                    public_key: sender_public_key,
                };
                let mut mempool = self.mempool.lock().unwrap();
                mempool.transactions.insert(signed_transaction.hash(), signed_transaction.clone());
                self.server.broadcast(Message::NewTransactionHashes(vec![signed_transaction.hash()]));
            }
            if theta != 0 {
                let interval = time::Duration::from_millis(theta as u64);
                thread::sleep(interval);
            }
        }
    }
}

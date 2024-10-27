use log::info;
use crate::types::hash::Hashable;
use std::time;
use std::thread;
use crate::network::message::Message;
use crate::network::server::Handle as ServerHandle;
use crate::types::transaction::generate_random_signed_transaction;
use crate::types::transaction::SignedTransaction;
use crate::types::mempool::Mempool;
use std::sync::{Arc, Mutex};
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};

#[derive(Clone)]
pub struct TransactionGenerator {
    server: ServerHandle,
    mempool: Arc<Mutex<Mempool>>,
}

impl TransactionGenerator {
    pub fn new(
        server: &ServerHandle,
        mempool: &Arc<Mutex<Mempool>>,
    ) -> Self {
        Self {
            server: server.clone(),
            mempool: Arc::clone(mempool),
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
        loop {
            let transaction = generate_random_signed_transaction();
            {
                let mut mempool = self.mempool.lock().unwrap();
                mempool.transactions.insert(transaction.hash(), transaction.clone());
            }
            self.server.broadcast(Message::NewTransactionHashes(vec![transaction.hash()]));
            if theta != 0 {
                let interval = time::Duration::from_millis(theta as u64);
                thread::sleep(interval);
            }
        }
    }
}

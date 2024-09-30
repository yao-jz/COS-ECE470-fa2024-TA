use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use log::{debug, info};
use crate::network::message::Message;
use crate::types::block::Block;
use crate::network::server::Handle as ServerHandle;
use crate::types::hash::{Hashable, H256};
use std::thread;
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;

#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_block_chan: Receiver<Block>,
    blockchain: Arc<Mutex<Blockchain>>,
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_block_chan: Receiver<Block>,
        blockchain: &Arc<Mutex<Blockchain>>,
    ) -> Self {
        Self {
            server: server.clone(),
            finished_block_chan,
            blockchain: Arc::clone(blockchain),
        }
    }

    pub fn start(self) {
        thread::Builder::new()
            .name("miner-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn worker_loop(&self) {
        loop {
            let _block = self.finished_block_chan.recv().expect("Receive finished block error");
            // TODO for student: insert this finished block to blockchain, and broadcast this block hash
            let finished_block = _block.clone();
            {
                let mut blockchain = self.blockchain.lock().unwrap();
                blockchain.insert(&finished_block);
            }
            let mut new_blocks: Vec<H256> = Vec::new();
            new_blocks.push(finished_block.hash());
            {println!("from miner/worker: insert new_blocks: {:?}", self.blockchain.lock().unwrap().tip());}
            self.server.broadcast(Message::NewBlockHashes(new_blocks));
        }
    }
}

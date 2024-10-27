use super::message::Message;
use super::peer;
use super::server::Handle as ServerHandle;
use crate::types::hash::{Hashable, H256};
use crate::blockchain::Blockchain;
use std::sync::{Arc, Mutex};
use crate::types::mempool::Mempool;
use crate::types::block::Block;
use crate::types::transaction::{verify, SignedTransaction};
use std::collections::HashMap;

use log::{debug, warn, error};

use std::thread;

#[cfg(any(test,test_utilities))]
use super::peer::TestReceiver as PeerTestReceiver;
#[cfg(any(test,test_utilities))]
use super::server::TestReceiver as ServerTestReceiver;
#[derive(Clone)]
pub struct Worker {
    msg_chan: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
}


impl Worker {
    pub fn new(
        num_worker: usize,
        msg_src: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
        server: &ServerHandle,
        blockchain: &Arc<Mutex<Blockchain>>,
        mempool: &Arc<Mutex<Mempool>>,
    ) -> Self {
        Self {
            msg_chan: msg_src,
            num_worker,
            server: server.clone(),
            blockchain: Arc::clone(blockchain),
            mempool: Arc::clone(mempool),
        }
    }

    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn worker_loop(&self) {
        loop {
            let mut block_buffer: HashMap<H256, Vec<Block>> = HashMap::new();
            let result = smol::block_on(self.msg_chan.recv());
            if let Err(e) = result {
                error!("network worker terminated {}", e);
                break;
            }
            let msg = result.unwrap();
            let (msg, mut peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }
                Message::NewBlockHashes(hashes) => {
                    debug!("NewBlockHashes: {:?}", hashes);
                    if hashes.len() > 0 {
                        peer.write(Message::GetBlocks(hashes));
                    }
                }
                Message::GetBlocks(hashes) => {
                    debug!("GetBlocks: {:?}", hashes);
                    let mut needed_blocks = vec![];
                    for hash in hashes {
                        let blockchain = self.blockchain.lock().unwrap();
                        let block = blockchain.blocks.get(&hash).cloned();
                        if let Some(block) = block {
                            needed_blocks.push(block);
                        }
                    }
                    if needed_blocks.len() > 0 {
                        peer.write(Message::Blocks(needed_blocks));
                    }
                }
                Message::Blocks(blocks) => {
                    debug!("Blocks: {:?}", blocks);
                    let mut new_blocks = vec![];
                    let mut needed_parent_blocks = vec![];
                    for block in blocks {
                        // check the difficulty
                        let difficulty = block.header.difficulty;
                        if block.hash() > difficulty {
                            continue; // ignore the block
                        }
                        let mut blockchain = self.blockchain.lock().unwrap();
                        let existing_block = blockchain.blocks.get(&block.hash()).cloned();
                        if let Some(existing_block) = existing_block {
                            continue;
                        } else {
                            let parent = blockchain.blocks.get(&block.header.parent).cloned();
                            if let Some(parent) = parent {
                                // check the same difficulty as parent
                                if block.header.difficulty != parent.header.difficulty {
                                    continue; // ignore the block
                                }
                                // check the signature
                                // check public key is the same as sender
                                // check nonce is increasing
                                // check sender has enough balance
                                // remove the transactions in the mempool
                                {
                                    let mut mempool = self.mempool.lock().unwrap();
                                    for tx in block.data.iter() {
                                        mempool.transactions.remove(&tx.hash());
                                    }
                                }
                                blockchain.insert(&block);
                                new_blocks.push(block.hash());
                                let mut new_block_hashes = vec![block.hash()];
                                loop {
                                    let mut b = block_buffer.remove(&new_block_hashes[0]);
                                    // remove the first element
                                    if let Some(b) = b {
                                        for this_block in b {
                                            // omit checking difficulty here
                                            // check the signature
                                            // check public key is the same as sender
                                            // check nonce is increasing
                                            // check sender has enough balance
                                            // remove the transactions in the mempool
                                            {
                                                let mut mempool = self.mempool.lock().unwrap();
                                                for tx in block.data.iter() {
                                                    mempool.transactions.remove(&tx.hash());
                                                }
                                            }
                                            blockchain.insert(&this_block);
                                            new_blocks.push(this_block.hash());
                                            new_block_hashes.push(this_block.hash());
                                        }
                                    }
                                    new_block_hashes = new_block_hashes[1..].to_vec();
                                    if new_block_hashes.len() == 0 {
                                        break;
                                    }
                                }
                            } else {
                                needed_parent_blocks.push(block.header.parent.clone());
                                let mut b = block_buffer.entry(block.header.parent.clone()).or_insert(vec![]);
                                b.push(block.clone());
                            }
                            
                        }
                    }
                    if new_blocks.len() > 0 {
                        self.server.broadcast(Message::NewBlockHashes(new_blocks));
                    }
                    if needed_parent_blocks.len() > 0 {
                        peer.write(Message::GetBlocks(needed_parent_blocks));
                    }
                }
                Message::NewTransactionHashes(hashes) => {
                    debug!("NewTransactionHashes: {:?}", hashes);
                    let mut needed_txs: Vec<H256> = vec![];
                    for hash in hashes {
                        let tx = self.mempool.lock().unwrap().transactions.get(&hash).cloned();
                        if let Some(tx) = tx {
                        } else {
                            needed_txs.push(hash);
                        }
                    }
                    if needed_txs.len() > 0 {
                        peer.write(Message::GetTransactions(needed_txs));
                    }
                }
                Message::GetTransactions(hashes) => {
                    debug!("GetTransactions: {:?}", hashes);
                    let mut needed_txs: Vec<SignedTransaction> = vec![];
                    for hash in hashes {
                        let tx = self.mempool.lock().unwrap().transactions.get(&hash).cloned();
                        if let Some(tx) = tx {
                            needed_txs.push(tx);
                        }
                    }
                    if needed_txs.len() > 0 {
                        peer.write(Message::Transactions(needed_txs));
                    }
                }
                Message::Transactions(txs) => {
                    debug!("Transactions: {:?}", txs);
                    // check the signature
                    let mut new_txs = vec![];
                    for tx in txs {
                        let valid = verify(&tx.transaction, &tx.public_key, &tx.signature);
                        if valid {
                            let mut mempool = self.mempool.lock().unwrap();
                            let existing_tx = mempool.transactions.get(&tx.hash()).cloned();
                            if let Some(existing_tx) = existing_tx {
                                continue;
                            } else {
                                mempool.transactions.insert(tx.hash(), tx.clone());
                                new_txs.push(tx.hash());
                            }
                        }
                    }
                    // check public key is the same as sender
                    // check nonce is increasing
                    // check sender has enough balance
                    
                    if new_txs.len() > 0 {
                        self.server.broadcast(Message::NewTransactionHashes(new_txs));
                    }
                }
            }
        }
    }
}

#[cfg(any(test,test_utilities))]
struct TestMsgSender {
    s: smol::channel::Sender<(Vec<u8>, peer::Handle)>
}
#[cfg(any(test,test_utilities))]
impl TestMsgSender {
    fn new() -> (TestMsgSender, smol::channel::Receiver<(Vec<u8>, peer::Handle)>) {
        let (s,r) = smol::channel::unbounded();
        (TestMsgSender {s}, r)
    }

    fn send(&self, msg: Message) -> PeerTestReceiver {
        let bytes = bincode::serialize(&msg).unwrap();
        let (handle, r) = peer::Handle::test_handle();
        smol::block_on(self.s.send((bytes, handle))).unwrap();
        r
    }
}
#[cfg(any(test,test_utilities))]
/// returns two structs used by tests, and an ordered vector of hashes of all blocks in the blockchain
fn generate_test_worker_and_start() -> (TestMsgSender, ServerTestReceiver, Vec<H256>) {
    let (server, server_receiver) = ServerHandle::new_for_test();
    let (test_msg_sender, msg_chan) = TestMsgSender::new();
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let all_blocks = blockchain.lock().unwrap().all_blocks_in_longest_chain();
    let mempool = Arc::new(Mutex::new(Mempool::new()));
    let worker = Worker::new(1, msg_chan, &server, &blockchain, &mempool);
    worker.start(); 
    (test_msg_sender, server_receiver, all_blocks)
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use ntest::timeout;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    use super::super::message::Message;
    use super::generate_test_worker_and_start;

    #[test]
    #[timeout(60000)]
    fn reply_new_block_hashes() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut peer_receiver = test_msg_sender.send(Message::NewBlockHashes(vec![random_block.hash()]));
        let reply = peer_receiver.recv();
        if let Message::GetBlocks(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_get_blocks() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let h = v.last().unwrap().clone();
        let mut peer_receiver = test_msg_sender.send(Message::GetBlocks(vec![h.clone()]));
        let reply = peer_receiver.recv();
        if let Message::Blocks(v) = reply {
            assert_eq!(1, v.len());
            assert_eq!(h, v[0].hash())
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_blocks() {
        let (test_msg_sender, server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut _peer_receiver = test_msg_sender.send(Message::Blocks(vec![random_block.clone()]));
        let reply = server_receiver.recv().unwrap();
        if let Message::NewBlockHashes(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
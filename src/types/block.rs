use serde::{Serialize, Deserialize};
use std::time::{self, UNIX_EPOCH};
use crate::types::hash::{H256, Hashable};
use crate::types::merkle::MerkleTree;
use crate::types::transaction::{SignedTransaction, generate_random_signed_transaction};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub data: Vec<SignedTransaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    pub parent: H256,
    pub nonce: u32,
    pub difficulty: H256,
    pub timestamp: u128,
    pub merkle_root: H256,
}

impl Hashable for BlockHeader {
    fn hash(&self) -> H256 {
        let data = bincode::serialize(self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &data).into()
    }
}   

impl Hashable for Block {
    fn hash(&self) -> H256 {
        self.header.hash()
    }
}

impl Block {
    pub fn get_parent(&self) -> H256 {
        self.header.parent
    }

    pub fn get_difficulty(&self) -> H256 {
        self.header.difficulty
    }
}

pub fn generate_block(parent: &H256, difficulty: &H256, signed_txs: Vec<SignedTransaction>) -> Block {
    let timestamp = std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let signed_txs_copy = signed_txs.clone();
    let root = MerkleTree::new(&signed_txs_copy).root();
    let nonce: u32 = rand::random();
    let head= BlockHeader{parent: *parent, nonce: nonce, difficulty: *difficulty, timestamp: timestamp, merkle_root: root};
    let block = Block{header: head, data: signed_txs_copy};
    block
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_block(parent: &H256) -> Block {
    use rand::{thread_rng, Rng};

    let mut rng = thread_rng();
    let tree = MerkleTree::new(&Vec::<SignedTransaction>::new());

    let header = BlockHeader {
        parent: parent.clone(),
        nonce: rng.gen::<u32>(),
        difficulty: H256::from([255; 32]),
        timestamp: std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
        merkle_root: tree.root(),
        
    };

    Block{header, data: vec![generate_random_signed_transaction()]}
}
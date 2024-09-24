use serde::{Serialize, Deserialize};
use std::time;
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
    pub timestamp: time::SystemTime,
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

#[cfg(any(test, test_utilities))]
pub fn generate_random_block(parent: &H256) -> Block {
    use rand::{thread_rng, Rng};

    let mut rng = thread_rng();
    let tree = MerkleTree::new(&Vec::<SignedTransaction>::new());

    let header = BlockHeader {
        parent: parent.clone(),
        nonce: rng.gen::<u32>(),
        difficulty: H256::from([255; 32]),
        timestamp: time::SystemTime::now(),
        merkle_root: tree.root(),
        
    };

    Block{header, data: vec![generate_random_signed_transaction()]}
}
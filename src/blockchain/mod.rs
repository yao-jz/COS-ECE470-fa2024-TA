use crate::types::block::{Block, BlockHeader};
use crate::types::merkle::MerkleTree;
use crate::types::state::State;
use crate::types::hash::Hashable;
use std::collections::HashMap;
use std::hash::Hash;
use std::time;
use crate::types::transaction::{SignedTransaction};
use crate::types::hash::H256;

pub struct Blockchain {
    pub blocks: HashMap<H256, Block>,
    pub heights: HashMap<H256, u32>,
    pub transactions: HashMap<H256, SignedTransaction>,
    pub states: HashMap<H256, State>,
    pub tip: H256,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let mut blocks: HashMap<H256, Block> = HashMap::new();
        let parent: H256 = H256::from([0; 32]);
        let nonce: u32 = 0;
        let mut easy: [u8; 32] = [0; 32];
        easy[0] = 0x04;
        let difficulty = H256::from(easy);
        let empty: Vec<H256> = Vec::new();
        let merkle_root = MerkleTree::new(&empty).root();
        let header = BlockHeader {
            parent,
            nonce,
            difficulty,
            timestamp: 0,
            merkle_root,
        };
        let genesis = Block {
            header,
            data: Vec::new(),
        };
        let genesis_hash = genesis.hash();
        println!("genesis hash: {:?}", genesis.hash());
        blocks.insert(genesis_hash, genesis);
        let mut heights: HashMap<H256, u32> = HashMap::new();
        heights.insert(genesis_hash, 0);
        let mut states: HashMap<H256, State> = HashMap::new();
        let transactions: HashMap<H256, SignedTransaction> = HashMap::new();
        let state = State::ico();
        states.insert(genesis_hash, state);
        Self {
            blocks,
            heights,
            transactions,
            states,
            tip: genesis_hash,
        }
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let parent = block.get_parent();

        let block_hash = block.hash();
        self.blocks.insert(block_hash, block.clone());
        let height = self.heights.get(&parent).unwrap() + 1;
        self.heights.insert(block_hash, height);
        let tip_height = self.heights.get(&self.tip).unwrap();
        if height > *tip_height {
            self.tip = block_hash;
        }

        // Update state
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        self.tip.clone()
    }

    // Get the difficulty
    pub fn get_difficulty(&self) -> H256 {
        let block = self.blocks.get(&self.tip).unwrap();
        block.header.difficulty.clone()
    }

    /// Get all blocks' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        // unimplemented!()
        let mut blocks = Vec::new();
        let mut current = self.tip.clone();
        while current != H256::from([0; 32]) {
            blocks.push(current.clone());
            let block = self.blocks.get(&current).unwrap();
            current = block.get_parent();
        }
        blocks.reverse();
        blocks
    }

    pub fn all_transactions_in_longest_chain(&self) -> Vec<Vec<H256>> {
        let mut transactions = Vec::new();
        let blocks = self.all_blocks_in_longest_chain();
        for block in blocks.iter() {
            let block = self.blocks.get(block).unwrap();
            let mut txs = Vec::new();
            for tx in block.data.iter() {
                txs.push(tx.hash());
            }
            transactions.push(txs);
        }
        transactions
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());

    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
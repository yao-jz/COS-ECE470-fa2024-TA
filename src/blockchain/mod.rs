use crate::types::block::{Block, BlockHeader};
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
        let genesis = H256::from([0; 32]);
        let blocks: HashMap<H256, Block> = HashMap::new();
        let mut heights: HashMap<H256, u32> = HashMap::new();
        let mut states: HashMap<H256, State> = HashMap::new();
        let transactions: HashMap<H256, SignedTransaction> = HashMap::new();

        heights.insert(genesis.clone(), 0);
        let state = State::ico();
        states.insert(genesis.clone(), state);
        Self {
            blocks,
            heights,
            transactions,
            states,
            tip: genesis,
        }
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let parent = block.get_parent();
        let parent_state = self.states.get(&parent).unwrap();

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
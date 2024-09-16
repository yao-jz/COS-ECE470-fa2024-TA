use super::hash::{Hashable, H256};

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    tree: Vec<Vec<H256>>,
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self where T: Hashable, {
        if data.len() == 0 {
            let mut tree_base: Vec<H256> = Vec::new();
            let mut tree: Vec<Vec<H256>> = Vec::new();
            tree_base.push(H256::from([0; 32]));
            tree.push(tree_base);
            return MerkleTree{tree};
        }
        let mut tree_base: Vec<H256> = Vec::new();
        let mut tree: Vec<Vec<H256>> = Vec::new();
        for entry in data{
            tree_base.push(entry.hash());
        }
        if data.len() % 2 != 0 {
            let extra = &data[data.len()-1];
            tree_base.push(extra.hash());
        }
        assert_eq!(tree_base.len() % 2, 0);
        tree.push(tree_base);
        let mut current_level = &tree[0];
        loop {
            let mut next_level: Vec<H256> = Vec::new();
            let mut left = 0;
            let mut right = 1;
            for _ in 0..current_level.len()/2 {
                let left_data = <[u8;32]>::from(current_level[left]);
                let right_data = <[u8;32]>::from(current_level[right]);
                let mut data = [0;64];
                data[..32].copy_from_slice(&left_data);
                data[32..].copy_from_slice(&right_data);
                next_level.push(ring::digest::digest(&ring::digest::SHA256, &data).into());
                left += 2;
                right += 2;
            }
            if next_level.len() == 1 {
                tree.push(next_level);
                break;
            }
            if next_level.len() % 2 != 0 {
                let extra = next_level[next_level.len()-1];
                next_level.push(extra);
            }
            assert_eq!(next_level.len() % 2, 0);
            tree.push(next_level);
            current_level = &tree[tree.len()-1];
        }
        MerkleTree{tree}
    }

    pub fn root(&self) -> H256 {
        self.tree[self.tree.len()-1][0].clone()
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        let mut proof: Vec<H256> = Vec::new();
        let mut index = index;
        for level in &self.tree {
            if level.len() == 1 {
                break;
            }
            if index % 2 == 0 {
                proof.push(level[index+1].clone());
            } else {
                proof.push(level[index-1].clone());
            }
            index /= 2;
        }
        proof
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    let mut hash = datum.clone();
    let mut index = index;
    if leaf_size <= index {
        return false;
    }
    for proof in proof {
        if index % 2 == 0 {
            let mut data = [0;64];
            let left_data = <[u8;32]>::from(hash);
            let right_data = <[u8;32]>::from(proof);
            data[..32].copy_from_slice(&left_data);
            data[32..].copy_from_slice(&right_data);
            hash = ring::digest::digest(&ring::digest::SHA256, &data).into();
        } else {
            let mut data = [0;64];
            let left_data = <[u8;32]>::from(proof);
            let right_data = <[u8;32]>::from(hash);
            data[..32].copy_from_slice(&left_data);
            data[32..].copy_from_slice(&right_data);
            hash = ring::digest::digest(&ring::digest::SHA256, &data).into();
        }
        index /= 2;
    }
    hash == *root
}
// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use crate::types::hash::H256;
    use super::*;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    #[test]
    fn merkle_root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn merkle_proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn merkle_verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
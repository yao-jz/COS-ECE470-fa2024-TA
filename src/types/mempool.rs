use std::collections::HashMap;
use indexmap::IndexMap;

use super::{
    hash::{Hashable, H256},
    transaction::SignedTransaction,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone)]
pub struct Mempool {
    pub transactions: IndexMap<H256, SignedTransaction>,
}

impl Mempool {
    pub fn new() -> Self {
        Self{
            transactions: IndexMap::new(),
        }
    }
}

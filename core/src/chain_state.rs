use ahash::{AHashMap, AHashSet};

pub struct ChainState {
    pub balances: AHashMap<[u8; 32], f32>, // account public key, balance
    pub transactions: AHashMap<[u8; 32], AHashSet<[u8; 32]>>, // account public key, txid
}
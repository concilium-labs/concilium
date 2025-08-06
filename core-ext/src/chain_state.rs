use ahash::{AHashMap, AHashSet};
use concilium_core::chain_state::ChainState;

pub trait ChainStateSupport {
    fn new() -> ChainState;
    fn get_balances(&self) -> &AHashMap<[u8; 32], f32>;
    fn get_transactions(&self) -> &AHashMap<[u8; 32], AHashSet<[u8; 32]>>;
    fn get_mut_balances(&mut self) -> &mut AHashMap<[u8; 32], f32>;
    fn get_mut_transactions(&mut self) -> &mut AHashMap<[u8; 32], AHashSet<[u8; 32]>>;
}

impl ChainStateSupport for ChainState {
    fn new() -> ChainState {
        Self {
            balances: AHashMap::new(),
            transactions: AHashMap::new(),
        }
    }

    fn get_balances(&self) -> &AHashMap<[u8; 32], f32> {
        &self.balances
    }

    fn get_transactions(&self) -> &AHashMap<[u8; 32], AHashSet<[u8; 32]>> {
        &self.transactions
    }
    
    fn get_mut_balances(&mut self) -> &mut AHashMap<[u8; 32], f32> {
        &mut self.balances
    }

    fn get_mut_transactions(&mut self) -> &mut AHashMap<[u8; 32], AHashSet<[u8; 32]>> {
        &mut self.transactions
    }
}
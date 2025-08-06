use std::sync::Arc;
use ahash::AHashMap;
use crate::node::ActiveNode;

#[derive(Clone)]
pub struct ActiveNodes {    
    pub by_public_key: AHashMap<[u8; 48], Arc<ActiveNode>>,
    pub by_id: AHashMap<u32, Arc<ActiveNode>>,
}
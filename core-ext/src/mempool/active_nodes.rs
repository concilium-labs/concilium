use std::sync::Arc;
use ahash::AHashMap;
use concilium_core::{active_nodes::ActiveNodes, node::ActiveNode};
use crate::node::active_node::ActiveNodeSupport;

pub trait ActiveNodesSupport {
    fn new() -> ActiveNodes;
    fn insert_or_update(&mut self, node: Arc<ActiveNode>) -> Option<Arc<ActiveNode>>;
    fn remove_by_public_key(&mut self, key: &[u8; 48]) -> Option<Arc<ActiveNode>>;
    fn remove_by_id(&mut self, key: u32) -> Option<Arc<ActiveNode>>;
    fn get_by_public_key(&self, key: &[u8; 48]) -> Option<&Arc<ActiveNode>>;
    fn get_by_id(&self, key: u32) -> Option<&Arc<ActiveNode>>;
    fn get_nodes_by_public_key(&self) -> &AHashMap<[u8; 48], Arc<ActiveNode>>;
    fn get_nodes_by_id(&self) -> &AHashMap<u32, Arc<ActiveNode>>;
    fn get_mut_nodes_by_public_key(&mut self) -> &AHashMap<[u8; 48], Arc<ActiveNode>>;
    fn get_mut_nodes_by_id(&mut self) -> &AHashMap<u32, Arc<ActiveNode>>;
    fn get_last_id(&self) -> Option<u32>;
    fn set_nodes_by_public_key(&mut self, nodes: &AHashMap<[u8; 48], Arc<ActiveNode>>);    
    fn set_nodes_by_id(&mut self, nodes: &AHashMap<u32, Arc<ActiveNode>>);                
}

impl ActiveNodesSupport for ActiveNodes {
    fn new() -> Self {
        Self {
            by_id: AHashMap::new(),
            by_public_key: AHashMap::new(),
        }
    }
    fn insert_or_update(&mut self, node: Arc<ActiveNode>) -> Option<Arc<ActiveNode>> {
        self.by_public_key.insert(node.get_public_key().clone(), Arc::clone(&node));
        self.by_id.insert(node.get_id(), Arc::clone(&node))
    }

    fn remove_by_public_key(&mut self, key: &[u8; 48]) -> Option<Arc<ActiveNode>> {
        if let Some(node) = self.by_public_key.remove(key) {
            self.by_id.remove(&node.get_id())
        } else {
            None
        }
    }
    
    fn remove_by_id(&mut self, key: u32) -> Option<Arc<ActiveNode>> {
        if let Some(node) = self.by_id.remove(&key) {
            self.by_public_key.remove(node.get_public_key())
        } else {
            None
        }
    }

    fn get_by_public_key(&self, key: &[u8; 48]) -> Option<&Arc<ActiveNode>> {
        self.by_public_key.get(key)
    }
    
    fn get_by_id(&self, key: u32) -> Option<&Arc<ActiveNode>> {
        self.by_id.get(&key)
    }
   
    fn get_nodes_by_public_key(&self) -> &AHashMap<[u8; 48], Arc<ActiveNode>> {
        &self.by_public_key
    }
    
    fn get_nodes_by_id(&self) -> &AHashMap<u32, Arc<ActiveNode>> {
        &self.by_id
    }
    
    fn get_mut_nodes_by_public_key(&mut self) -> &AHashMap<[u8; 48], Arc<ActiveNode>> {
        &mut self.by_public_key
    }
    
    fn get_mut_nodes_by_id(&mut self) -> &AHashMap<u32, Arc<ActiveNode>> {
        &mut self.by_id
    }

    fn get_last_id(&self) -> Option<u32> {
        self.by_public_key.iter().map(|(_, node)| node.get_id()).max()
    }
    
    fn set_nodes_by_public_key(&mut self, nodes: &AHashMap<[u8; 48], Arc<ActiveNode>>) {        
        let mut by_public_key = AHashMap::new();
        let mut by_id = AHashMap::new();
        
        nodes.iter().for_each(|(_, node)| {
            by_public_key.insert(node.get_public_key().clone(), Arc::clone(node));
            by_id.insert(node.get_id(), Arc::clone(node));
        });

        self.by_public_key = by_public_key;
        self.by_id= by_id;
    }
    
    fn set_nodes_by_id(&mut self, nodes: &AHashMap<u32, Arc<ActiveNode>>) {
        let mut by_public_key = AHashMap::new();
        let mut by_id= AHashMap::new();
        
        nodes.iter().for_each(|(_, node)| {
            by_public_key.insert(node.get_public_key().clone(), Arc::clone(node));
            by_id.insert(node.get_id(), Arc::clone(node));
        });

        self.by_public_key = by_public_key;
        self.by_id = by_id;
    }
}
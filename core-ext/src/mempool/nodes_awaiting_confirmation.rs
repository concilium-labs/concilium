use ahash::AHashMap;
use concilium_core::{
    node::AwaitingConfirmationNode, 
    nodes_awaiting_confirmation::NodesAwaitingConfirmation
};
use crate::node::awaiting_confirmation_node::AwaitingConfirmationNodeSupport;

pub trait NodesAwaitingConfirmationSupport {
    fn new() -> NodesAwaitingConfirmation;
    fn insert_or_update(&mut self, node: AwaitingConfirmationNode) -> Option<AwaitingConfirmationNode>;
    fn set_nodes(&mut self, nodes: AHashMap<[u8; 48], AwaitingConfirmationNode>);
    fn remove(&mut self, key: &[u8; 48]) -> Option<AwaitingConfirmationNode>;
    fn get(&self, key: &[u8; 48]) -> Option<&AwaitingConfirmationNode>;
    fn get_mut(&mut self, key: &[u8; 48]) -> Option<&mut AwaitingConfirmationNode>;
    fn get_nodes(&self) -> &AHashMap<[u8; 48], AwaitingConfirmationNode>;
    fn get_self(&self) -> &NodesAwaitingConfirmation;
    fn get_mut_nodes(&mut self) -> &mut AHashMap<[u8; 48], AwaitingConfirmationNode>;
    fn len(&self) -> usize;
    fn get_last_id(&self) -> Option<u32>;    
}

impl NodesAwaitingConfirmationSupport for NodesAwaitingConfirmation {
    fn new() -> Self {
        Self {
            nodes: AHashMap::new()
        }
    }

    fn insert_or_update(&mut self, node: AwaitingConfirmationNode) -> Option<AwaitingConfirmationNode> {
        self.nodes.insert(node.get_public_key().clone(), node)
    }
    
    fn set_nodes(&mut self, nodes: AHashMap<[u8; 48], AwaitingConfirmationNode>) {
        self.nodes = nodes;
    }

    fn remove(&mut self, key: &[u8; 48]) -> Option<AwaitingConfirmationNode> {
        self.nodes.remove(key)
    }

    fn get(&self, key: &[u8; 48]) -> Option<&AwaitingConfirmationNode> {
        self.nodes.get(key)
    }
    
    fn get_mut(&mut self, key: &[u8; 48]) -> Option<&mut AwaitingConfirmationNode> {
        self.nodes.get_mut(key)
    }
    
    fn get_nodes(&self) -> &AHashMap<[u8; 48], AwaitingConfirmationNode> {
        &self.nodes
    }
    
    fn get_self(&self) -> &NodesAwaitingConfirmation {
        &self
    }
    
    fn get_mut_nodes(&mut self) -> &mut AHashMap<[u8; 48], AwaitingConfirmationNode> {
        &mut self.nodes
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn get_last_id(&self) -> Option<u32> {
        self.nodes.iter().map(|(_, node)| node.get_id()).max()
    }
}
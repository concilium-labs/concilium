use ahash::AHashMap;
use crate::node::AwaitingConfirmationNode;

pub struct NodesAwaitingConfirmation {
    pub nodes: AHashMap<[u8; 48], AwaitingConfirmationNode>
}

use std::sync::Arc;
use ahash::AHashMap;
use tokio::sync::{Mutex, RwLock};
use crate::{active_nodes::ActiveNodes, chain_state::ChainState, epoch::EpochPool, node::SelfNode, nodes_awaiting_confirmation::NodesAwaitingConfirmation, temporary_node_ids::TemporaryNodeIds, transaction::TXOutput};

pub struct Mempool {
    pub self_node: Arc<RwLock<SelfNode>>,
    pub active_nodes: Arc<RwLock<ActiveNodes>>,
    pub nodes_awaiting_confirmation: Arc<RwLock<NodesAwaitingConfirmation>>,
    pub epoch_pool: Arc<EpochPool>,
    pub temporary_node_ids: Arc<TemporaryNodeIds>, 
    pub utxos: Arc<RwLock<AHashMap<([u8; 32], usize), TXOutput>>>, // txid, vout index, TXOutput,
    pub chain_state: Arc<RwLock<ChainState>>,
    pub bootstrap_node_signature: Arc<Mutex<[u8; 96]>>
}
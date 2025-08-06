use std::{env, sync::Arc};
use active_nodes::ActiveNodesSupport;
use ahash::AHashMap;
use chrono::Utc;
use concilium_core::{
    active_nodes::ActiveNodes, chain_state::ChainState, epoch::EpochPool, mempool::Mempool, node::SelfNode, nodes_awaiting_confirmation::NodesAwaitingConfirmation, temporary_node_ids::TemporaryNodeIds, transaction::TXOutput
};
use concilium_error::Error;
use concilium_shared::ip::ipv4_to_array;
use nodes_awaiting_confirmation::NodesAwaitingConfirmationSupport;
use tokio::sync::{Mutex, RwLock};
use crate::{
    chain_state::ChainStateSupport, epoch::EpochPoolSupport, node::self_node::SelfNodeSupport, temporary_node_ids::TemporaryNodeIdsSupport
};

pub mod active_nodes;
pub mod nodes_awaiting_confirmation;

pub trait MempoolSupport {
    fn new() -> Result<Mempool, Error>;  
    fn get_self_node(&self) -> Arc<RwLock<SelfNode>>;
    fn get_active_nodes(&self) -> Arc<RwLock<ActiveNodes>>;
    fn get_nodes_awaiting_confirmation(&self) -> Arc<RwLock<NodesAwaitingConfirmation>>;
    fn get_epoch_pool(&self) -> Arc<EpochPool>;
    fn get_temporary_node_ids(&self) -> Arc<TemporaryNodeIds>;
    fn get_utxos(&self) -> Arc<RwLock<AHashMap<([u8; 32], usize), TXOutput>>>;
    fn get_chain_state(&self) -> Arc<RwLock<ChainState>>;
    fn get_bootstrap_node_signature(&self) -> Arc<Mutex<[u8; 96]>>;
}

impl MempoolSupport for Mempool {
    fn new() -> Result<Self, Error> {
        Ok(
            Self {
                self_node: Arc::new(RwLock::new(
                        SelfNode::new(
                            0, 
                            env::var("NODE_NAME")?.as_bytes().to_vec(), 
                            hex::decode(env::var("NODE_PUBLIC_KEY")?.trim())?.try_into()?, 
                            hex::decode(env::var("NODE_PRIVATE_KEY")?.trim())?.try_into()?, 
                            ipv4_to_array(env::var("NODE_IP_ADDRESS")?.trim())?, 
                            env::var("NODE_RPC_PORT")?.parse()?, 
                            env::var("APP_VERSION")?.trim().as_bytes().to_vec(), 
                            Utc::now().timestamp()
                        )
                    )),
                active_nodes: Arc::new(RwLock::new(ActiveNodes::new())),
                nodes_awaiting_confirmation: Arc::new(RwLock::new(NodesAwaitingConfirmation::new())),
                epoch_pool: Arc::new(EpochPool::new()),
                temporary_node_ids: Arc::new(TemporaryNodeIds::new()),
                utxos: Arc::new(RwLock::new(AHashMap::new())),
                chain_state: Arc::new(RwLock::new(ChainState::new())),
                bootstrap_node_signature: Arc::new(Mutex::new([0; 96]))
            }
        )
    }

    fn get_self_node(&self) -> Arc<RwLock<SelfNode>> {
        Arc::clone(&self.self_node)
    }
    
    fn get_active_nodes(&self) -> Arc<RwLock<ActiveNodes>> {
        Arc::clone(&self.active_nodes)
    }
    
    fn get_nodes_awaiting_confirmation(&self) -> Arc<RwLock<NodesAwaitingConfirmation>> {
        Arc::clone(&self.nodes_awaiting_confirmation)
    }
    
    fn get_epoch_pool(&self) -> Arc<EpochPool> {
        Arc::clone(&self.epoch_pool)
    }
    
    fn get_temporary_node_ids(&self) -> Arc<TemporaryNodeIds> {
        Arc::clone(&self.temporary_node_ids)
    }
    
    fn get_utxos(&self) -> Arc<RwLock<AHashMap<([u8; 32], usize), TXOutput>>> {
        Arc::clone(&self.utxos)
    }

    fn get_chain_state(&self) -> Arc<RwLock<ChainState>> {
        Arc::clone(&self.chain_state)
    }
    
    fn get_bootstrap_node_signature(&self) -> Arc<Mutex<[u8; 96]>> {
        Arc::clone(&self.bootstrap_node_signature)
    }
}
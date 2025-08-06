use std::sync::Arc;
use ahash::AHashMap;
use tokio::{sync::{mpsc::Sender, oneshot, RwLock}, task::JoinHandle};
use tonic::transport::Channel;
use concilium_proto_defs::transaction::{
    transaction_client::TransactionClient, AccreditationCouncilRequest, AccreditationCouncilResponse, BroadcastRequest, BroadcastResponse, LeaderRequest, LeaderResponse, SaveRequest, SaveResponse
};
use crate::{db::DB, mempool::Mempool};

#[derive(Debug)]
pub struct Client {
    pub client: TransactionClient<Channel>,
    
    pub leader_tx: Arc<Sender<LeaderRequest>>,
    pub leader_response_state: Arc<RwLock<AHashMap<[u8; 16], oneshot::Sender<LeaderResponse>>>>,
    pub leader_thread_handler: JoinHandle<()>,
    
    pub accreditation_council_tx: Arc<Sender<AccreditationCouncilRequest>>,
    pub accreditation_council_response_state: Arc<RwLock<AHashMap<[u8; 16], oneshot::Sender<AccreditationCouncilResponse>>>>,
    pub accreditation_council_thread_handler: JoinHandle<()>,
    
    pub broadcast_tx: Arc<Sender<BroadcastRequest>>,
    pub broadcast_response_state: Arc<RwLock<AHashMap<[u8; 16], oneshot::Sender<BroadcastResponse>>>>,
    pub broadcast_thread_handler: JoinHandle<()>,
    
    pub save_tx: Arc<Sender<SaveRequest>>,
    pub save_response_state: Arc<RwLock<AHashMap<[u8; 16], oneshot::Sender<SaveResponse>>>>,
    pub save_thread_handler: JoinHandle<()>,
}

pub struct Server {
    pub mempool: Arc<Mempool>,
    pub db: Arc<DB>,
}
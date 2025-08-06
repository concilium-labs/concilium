use std::sync::Arc;
use tokio::{sync::mpsc::Sender, task::JoinHandle};
use tonic::transport::Channel;
use concilium_proto_defs::epoch::{
    epoch_client::EpochClient,
    InitialRequest, 
    SyncRequest
};
use crate::mempool::Mempool;

#[derive(Debug)]
pub struct Client {
    pub client: EpochClient<Channel>,
    pub initial_tx: Arc<Sender<InitialRequest>>,
    pub sync_tx: Arc<Sender<SyncRequest>>,
    pub initial_thread_handler: JoinHandle<()>,
    pub sync_thread_handler: JoinHandle<()>
}

pub struct Server {
    pub mempool: Arc<Mempool>
}

use std::sync::Arc;
use ahash::AHashMap;
use concilium_core::rpc::transaction::Client;
use rand::random;
use tokio::{
    sync::{mpsc, oneshot::{self, Receiver}, RwLock},
    task::JoinHandle
};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{transport::Channel, Streaming};
use concilium_log as log;
use concilium_error::Error;
use concilium_proto_defs::transaction::{
    transaction_client::TransactionClient, AccreditationCouncilRequest, AccreditationCouncilResponse, BroadcastRequest, BroadcastResponse, LeaderRequest, LeaderResponse, SaveRequest, SaveResponse 
};

#[tonic::async_trait]
pub trait ClientSupport {
    async fn connect(dst: &str) -> Result<Client, Error>;
    fn get_client(&self) -> &TransactionClient<Channel>;
    
    fn get_leader_thread_handler(self) -> JoinHandle<()>;
    async fn leader_request(&self, node_id: u32, nonce: u64, transaction: &[u8]) -> Result<Receiver<LeaderResponse>, Error>;
    
    fn get_accreditation_council_thread_handler(self) -> JoinHandle<()>;
    async fn accreditation_council_request(&self, node_id: u32, nonce: u64, transaction: &[u8]) -> Result<Receiver<AccreditationCouncilResponse>, Error>;
    
    fn get_broadcast_thread_handler(self) -> JoinHandle<()>;
    async fn broadcast_request(&self, node_id: u32, nonce: u64, broadcast_transaction_temp: &[u8]) -> Result<Receiver<BroadcastResponse>, Error>;
    
    fn get_save_thread_handler(self) -> JoinHandle<()>;
    async fn save_request(&self, node_id: u32, nonce: u64, transaction: &[u8]) -> Result<Receiver<SaveResponse>, Error>;
}

#[tonic::async_trait]
impl ClientSupport for Client {
    async fn connect(dst: &str) -> Result<Self, Error> {
        let mut client = TransactionClient::connect(format!("http://{}", dst))
        .await?;

        let leader_response_state: Arc<RwLock<AHashMap<[u8; 16], oneshot::Sender<LeaderResponse>>>> = Arc::new(RwLock::new(AHashMap::new()));
        let accreditation_council_response_state: Arc<RwLock<AHashMap<[u8; 16], oneshot::Sender<AccreditationCouncilResponse>>>> = Arc::new(RwLock::new(AHashMap::new()));
        let broadcast_response_state: Arc<RwLock<AHashMap<[u8; 16], oneshot::Sender<BroadcastResponse>>>> = Arc::new(RwLock::new(AHashMap::new()));
        let save_response_state: Arc<RwLock<AHashMap<[u8; 16], oneshot::Sender<SaveResponse>>>> = Arc::new(RwLock::new(AHashMap::new()));
        
        let (leader_tx, leader_rx) = mpsc::channel::<LeaderRequest>(u16::MAX as usize);
        let (accreditation_council_tx, accreditation_council_rx) = mpsc::channel::<AccreditationCouncilRequest>(u16::MAX as usize);
        let (broadcast_tx, broadcast_rx) = mpsc::channel::<BroadcastRequest>(u16::MAX as usize);
        let (save_tx, save_rx) = mpsc::channel::<SaveRequest>(u16::MAX as usize);

        let leader_receiver = ReceiverStream::new(leader_rx);
        let accreditation_council_receiver = ReceiverStream::new(accreditation_council_rx);
        let broadcast_receiver = ReceiverStream::new(broadcast_rx);
        let save_receiver = ReceiverStream::new(save_rx);
    
        let leader_response = client
        .leader(tonic::Request::new(leader_receiver))
        .await?;
        
        let accreditation_council_response = client
        .accreditation_council(tonic::Request::new(accreditation_council_receiver))
        .await?;
        
        let broadcast_response = client
        .broadcast(tonic::Request::new(broadcast_receiver))
        .await?;
        
        let save_response = client
        .save(tonic::Request::new(save_receiver))
        .await?;

        let mut leader_stream: Streaming<LeaderResponse> = leader_response.into_inner();
        let mut accreditation_council_stream: Streaming<AccreditationCouncilResponse> = accreditation_council_response.into_inner();
        let mut broadcast_stream: Streaming<BroadcastResponse> = broadcast_response.into_inner();
        let mut save_stream: Streaming<SaveResponse> = save_response.into_inner();

        let leader_response_state_clone = Arc::clone(&leader_response_state);
        let leader_thread_handler = tokio::spawn(async move {
            while let Some(response) = leader_stream.next().await {
                match response {
                    Ok(data) => {
                        if let Ok(request_id) = data.request_id.clone().try_into() {
                            let mut state = leader_response_state_clone.write().await;
                            if let Some(tx) = state.remove(&request_id) {
                                tx.send(data).ok();   
                            } else {
                                let (node_id, nonce, random_number) = load_u32_pair(request_id);
                                println!("request_id: {}-{}-{} not found", node_id, nonce, random_number);
                            }
                        }                        
                    },
                    Err(e) => {
                        log::error(e.message().to_string().as_str()).await.ok();
                    }
                }
            }
        });
        
        let accreditation_council_response_state_clone = Arc::clone(&accreditation_council_response_state);
        let accreditation_council_thread_handler = tokio::spawn(async move {
            while let Some(response) = accreditation_council_stream.next().await {
                match response {
                    Ok(data) => {
                        if let Ok(request_id) = data.request_id.clone().try_into() {
                            let mut state = accreditation_council_response_state_clone.write().await;
                            if let Some(tx) = state.remove(&request_id) {
                                tx.send(data).ok();   
                            } else {
                                let (node_id, nonce, random_number) = load_u32_pair(request_id);
                                println!("request_id: {}-{}-{} not found", node_id, nonce, random_number);                        
                            }
                        }
                    },
                    Err(e) => {
                        log::error(e.message().to_string().as_str()).await.ok();
                    }
                }
            }
        });
        
        let broadcast_response_state_clone = Arc::clone(&broadcast_response_state);
        let broadcast_thread_handler = tokio::spawn(async move {
            while let Some(response) = broadcast_stream.next().await {
                match response {
                    Ok(data) => {
                        if let Ok(request_id) = data.request_id.clone().try_into() {
                            let mut state = broadcast_response_state_clone.write().await;
                            if let Some(tx) = state.remove(&request_id) {
                                tx.send(data).ok();   
                            } else {
                                let (node_id, nonce, random_number) = load_u32_pair(request_id);
                                println!("request_id: {}-{}-{} not found", node_id, nonce, random_number);
                            }
                        }                        
                    },
                    Err(e) => {
                        log::error(e.message().to_string().as_str()).await.ok();
                    }
                }
            }
        });
        
        let save_response_state_clone = Arc::clone(&save_response_state);
        let save_thread_handler = tokio::spawn(async move {
            while let Some(response) = save_stream.next().await {
                match response {
                    Ok(data) => {
                        if let Ok(request_id) = data.request_id.clone().try_into() {
                            let mut state = save_response_state_clone.write().await;
                            if let Some(tx) = state.remove(&request_id) {
                                tx.send(data).ok();   
                            } else {
                                let (node_id, nonce, random_number) = load_u32_pair(request_id);
                                println!("request_id: {}-{}-{} not found", node_id, nonce, random_number);
                            }
                        }                        
                    },
                    Err(e) => {
                        log::error(e.message().to_string().as_str()).await.ok();
                    }
                }
            }
        });

        Ok(
            Self {
                client,

                leader_tx: Arc::new(leader_tx),
                leader_response_state,
                leader_thread_handler,
        
                accreditation_council_tx: Arc::new(accreditation_council_tx),
                accreditation_council_response_state,
                accreditation_council_thread_handler,
        
                broadcast_tx: Arc::new(broadcast_tx),
                broadcast_response_state,
                broadcast_thread_handler,
                
                save_tx: Arc::new(save_tx),
                save_response_state,
                save_thread_handler,
            }
        )
    }

    fn get_client(&self) -> &TransactionClient<Channel> {
        &self.client
    }

    fn get_leader_thread_handler(self) -> JoinHandle<()> {
        self.leader_thread_handler
    }

    async fn leader_request(&self, node_id: u32, nonce: u64, transaction: &[u8]) -> Result<Receiver<LeaderResponse>, Error> {
        let (tx, rx) = oneshot::channel();
        let request_id = store_u32_pair(node_id, nonce, random::<u32>());

        {
            let mut state = self.leader_response_state.write().await;
            state.insert(request_id, tx);
        }

        self.leader_tx.send(
            LeaderRequest { 
                request_id: request_id.to_vec(),
                transaction: transaction.to_vec()
            }
        ).await?;
        
        Ok(rx)
    }

    fn get_accreditation_council_thread_handler(self) -> JoinHandle<()> {
        self.accreditation_council_thread_handler
    }
    
    async fn accreditation_council_request(&self, node_id: u32, nonce: u64, transaction: &[u8]) -> Result<Receiver<AccreditationCouncilResponse>, Error> {
        let (tx, rx) = oneshot::channel();
        let request_id = store_u32_pair(node_id, nonce, random::<u32>());

        {
            let mut state = self.accreditation_council_response_state.write().await;
            state.insert(request_id, tx);
        }

        self.accreditation_council_tx.send(
            AccreditationCouncilRequest { 
                request_id: request_id.to_vec(),
                transaction: transaction.to_vec()
            }
        ).await?;
        
        Ok(rx)
    }
    
    fn get_broadcast_thread_handler(self) -> JoinHandle<()> {
        self.broadcast_thread_handler
    }
    
    async fn broadcast_request(&self, node_id: u32, nonce: u64, broadcast_transaction_temp: &[u8]) -> Result<Receiver<BroadcastResponse>, Error> {
        let (tx, rx) = oneshot::channel();
        let request_id = store_u32_pair(node_id, nonce, random::<u32>());

        {
            let mut state = self.broadcast_response_state.write().await;
            state.insert(request_id, tx);
        }

        self.broadcast_tx.send(
            BroadcastRequest { 
                request_id: request_id.to_vec(),
                broadcast_transaction_temp: broadcast_transaction_temp.to_vec()
            }
        ).await?;
        
        Ok(rx)
    }
    
    fn get_save_thread_handler(self) -> JoinHandle<()> {
        self.save_thread_handler
    }
    
    async fn save_request(&self, node_id: u32, nonce: u64, transaction: &[u8]) -> Result<Receiver<SaveResponse>, Error> {
        let (tx, rx) = oneshot::channel();
        let request_id = store_u32_pair(node_id, nonce, random::<u32>());

        {
            let mut state = self.save_response_state.write().await;
            state.insert(request_id, tx);
        }

        self.save_tx.send(
            SaveRequest { 
                request_id: request_id.to_vec(),
                transaction: transaction.to_vec()
            }
        ).await?;
        
        Ok(rx)
    }
}

fn store_u32_pair(node_id: u32, nonce: u64, random_number: u32) -> [u8; 16] {
    let mut bytes = [0u8; 16];

    bytes[..4].copy_from_slice(&node_id.to_le_bytes());
    bytes[4..12].copy_from_slice(&nonce.to_le_bytes());
    bytes[12..].copy_from_slice(&random_number.to_le_bytes());

    bytes
}

fn load_u32_pair(bytes: [u8; 16]) -> (u32, u64, u32) {
    let node_id = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let nonce = u64::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11]]);
    let random_number = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);

    (node_id, nonce, random_number)
}
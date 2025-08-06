use std::sync::Arc;
use concilium_core::rpc::epoch::Client;
use tokio::{
    sync::mpsc,
    task::JoinHandle
};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{transport::Channel, Streaming};
use concilium_log as log;
use concilium_error::Error;
use concilium_proto_defs::epoch::{epoch_client::EpochClient, Response as EpochResponse, InitialRequest, SyncRequest};

#[tonic::async_trait]
pub trait ClientSupport {
    async fn connect(dst: &str) -> Result<Client, Error>;
    fn get_initial_thread_handler(self) -> JoinHandle<()>;
    fn get_sync_thread_handler(self) -> JoinHandle<()>;
    fn get_client(&self) -> &EpochClient<Channel>;
    async fn initial_request(&self, id: u64, random_data: u64) -> Result<(), Error>;
    async fn sync_request(&self, id: u64, hash: &[u8; 32]) -> Result<(), Error>;
}

#[tonic::async_trait]
impl ClientSupport for Client {
    async fn connect(dst: &str) -> Result<Self, Error> {
        let mut client = EpochClient::connect(format!("http://{}", dst))
        .await?;
        
        let (initial_tx, initial_rx) = mpsc::channel::<InitialRequest>(u16::MAX as usize);
        let (sync_tx, sync_rx) = mpsc::channel::<SyncRequest>(u16::MAX as usize);

        let initial_receiver= ReceiverStream::new(initial_rx);
        let sync_receiver= ReceiverStream::new(sync_rx);
    
        let initial_response = client
        .initial(tonic::Request::new(initial_receiver))
        .await?;
        
        let sync_response = client
        .sync(tonic::Request::new(sync_receiver))
        .await?;

        let mut initial_stream: Streaming<EpochResponse> = initial_response.into_inner();
        let mut sync_stream: Streaming<EpochResponse> = sync_response.into_inner();

        let initial_thread_handler = tokio::spawn(async move {
            while let Some(response) = initial_stream.next().await {
                match response {
                    Ok(_data) => {
                        // write code
                    },
                    Err(e) => {
                        log::error(e.message().to_string().as_str()).await.ok();
                    }
                }
            }
        });
        
        let sync_thread_handler = tokio::spawn(async move {
            while let Some(response) = sync_stream.next().await {
                match response {
                    Ok(_data) => {
                        // write code           
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
                initial_tx: Arc::new(initial_tx),
                sync_tx: Arc::new(sync_tx),
                initial_thread_handler,
                sync_thread_handler
            }
        )
    }

    fn get_initial_thread_handler(self) -> JoinHandle<()> {
        self.initial_thread_handler
    }
    
    fn get_sync_thread_handler(self) -> JoinHandle<()> {
        self.sync_thread_handler
    }

    fn get_client(&self) -> &EpochClient<Channel> {
        &self.client
    }

    async fn initial_request(&self, id: u64, random_data: u64) -> Result<(), Error> {
        Ok(
            self.initial_tx.send(
                InitialRequest { 
                    id, 
                    random_data 
                }
            ).await?
        )
    }
    
    async fn sync_request(&self, id: u64, hash: &[u8; 32]) -> Result<(), Error> {
        Ok(
            self.sync_tx.send(
                SyncRequest { 
                    id, 
                    hash: hash.to_vec()
                }
            ).await?
        )
    }
}
use concilium_core::{epoch::Epoch, mempool::Mempool, rpc::epoch::Server as OriginServer};
use concilium_core_ext::{epoch::{EpochSupport, EpochPoolSupport}, mempool::MempoolSupport};
use concilium_shared::epoch::current_epoch_number;
use tonic::{Request, Response, Streaming, Status};
use tokio_stream::{Stream, StreamExt, wrappers::ReceiverStream};
use std::{ops::Deref, pin::Pin, sync::Arc};
use tokio::sync::mpsc;
use concilium_log as log;
use concilium_proto_defs::epoch::{
    InitialRequest, 
    SyncRequest, 
    Response as EpochResponse, 
    epoch_server::Epoch as EpochServerSupport
};

pub struct Server(pub OriginServer);

impl Deref for Server {
    type Target = OriginServer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait ServerSupport {
    fn new(mempool: Arc<Mempool>) -> Self;
}

impl ServerSupport for Server {
    fn new(mempool: Arc<Mempool>) -> Self {
        Self(OriginServer { mempool })
    }
}

type ResponseStream = Pin<Box<dyn Stream<Item = Result<concilium_proto_defs::epoch::Response, Status>> + Send>>;
type ServiceResult<T> = Result<Response<T>, Status>;

#[tonic::async_trait]
impl EpochServerSupport for Server {
    type InitialStream = ResponseStream;
    type SyncStream = ResponseStream;

    async fn initial(&self, request: Request<Streaming<InitialRequest>>) -> ServiceResult<Self::InitialStream> {
        let (tx, rx) = mpsc::channel(u16::MAX as usize);
        let mut stream = request.into_inner();
        
        let mempool = Arc::clone(&self.mempool);
        tokio::spawn(async move {
            while let Some(request) = stream.next().await {
                match request {
                    Ok(data) => {
                        let current_epoch_number = u64::try_from(current_epoch_number()).unwrap();
                        if data.id == (current_epoch_number + 1) {
                            {
                                let epoch_pool_read = mempool.get_epoch_pool().get_read();

                                if let Some(epoch) = epoch_pool_read.get(&data.id) {
                                    let mut random_numbers = epoch.get_random_numbers().to_vec();
                                    random_numbers.push(data.random_data);

                                    let epoch_pool_write = mempool.get_epoch_pool().get_write();
                                    let mut epoch_pool_write_guard = epoch_pool_write.lock().await;

                                    epoch_pool_write_guard.update(epoch.get_id().clone(), Arc::new(Epoch::new(epoch.get_id().clone(), epoch.get_last_node_id().clone(), epoch.get_final_hash().clone(), random_numbers, epoch.get_hashes().clone())));
                                    epoch_pool_write_guard.publish();
                                }; 
                            }

                            tx.send(
                                Ok(
                                    EpochResponse {
                                        status: true
                                    }
                                )
                            ).await.unwrap();
                        } else {
                            tx.send(
                                Ok(
                                    EpochResponse {
                                        status: false
                                    }
                                )
                            ).await.unwrap();
                        }
                    },
                    Err(e) => {
                        log::error(format!("connect to network error: [{}]", e.message()).as_str()).await.ok();
                        break;
                    }
                }
            }
        });

        let out = ReceiverStream::new(rx);

        Ok(
            Response::new(Box::pin(out) as Self::InitialStream)
        )
    }
    
    async fn sync(&self, request: Request<Streaming<SyncRequest>>) -> ServiceResult<Self::SyncStream> {
        let (tx, rx) = mpsc::channel(u16::MAX as usize);
        let mut stream = request.into_inner();
        
        let mempool = Arc::clone(&self.mempool);
        tokio::spawn(async move {
            while let Some(request) = stream.next().await {
                match request {
                    Ok(data) => {
                        let current_epoch_number = u64::try_from(current_epoch_number()).unwrap();
                        if data.id == (current_epoch_number + 1) {
                            {
                                let epoch_pool_read = mempool.get_epoch_pool().get_read();
                                if let Some(epoch) = epoch_pool_read.get(&data.id) {
                                    let hash: [u8; 32] = data.hash.try_into().unwrap();                                
                                    
                                    let mut hashes = epoch.get_hashes().clone();
                                    if let Some(h) = hashes.get_mut(&hash) {                                
                                        *h += 1;
                                    } else {                                
                                        hashes.insert(hash, 1);
                                    }

                                    let epoch_pool_write = mempool.get_epoch_pool().get_write();
                                    let mut epoch_pool_write_guard = epoch_pool_write.lock().await;

                                    epoch_pool_write_guard.update(epoch.get_id().clone(), Arc::new(Epoch::new(epoch.get_id().clone(), epoch.get_last_node_id().clone(), epoch.get_final_hash().clone(), epoch.get_random_numbers().to_vec(), hashes)));
                                    epoch_pool_write_guard.publish();
                                };
                            }

                            tx.send(
                                Ok(
                                    EpochResponse {
                                        status: true
                                    }
                                )
                            ).await.unwrap();
                        } else {
                            tx.send(
                                Ok(
                                    EpochResponse {
                                        status: false
                                    }
                                )
                            ).await.unwrap();
                        }
                    },
                    Err(e) => {
                        log::error(format!("connect to network error: [{}]", e.message()).as_str()).await.ok();
                        break;
                    }
                }
            }
        });

        let out = ReceiverStream::new(rx);

        Ok(
            Response::new(Box::pin(out) as Self::SyncStream)
        )
    }
}
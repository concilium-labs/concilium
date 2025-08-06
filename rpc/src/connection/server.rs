use std::{ops::Deref, sync::Arc};
use ahash::AHashMap;
use concilium_core_ext::{epoch::{EpochSupport, EpochPoolSupport}, mempool::{active_nodes::ActiveNodesSupport, MempoolSupport}, node::{active_node::ActiveNodeSupport, awaiting_confirmation_node::AwaitingConfirmationNodeSupport, self_node::SelfNodeSupport}};
use tokio::time::Duration;
use concilium_core::{
    epoch::Epoch, 
    mempool::Mempool, 
    node::{ActiveNode, AwaitingConfirmationNode}, 
    rpc::{
        connection::{
            Client as ConnectionClient, Server as OriginServer
        },
        epoch::Client as EpochClient,
        transaction::Client as TransactionClient
    }
};
use tokio::time::sleep;
use tonic::{Request, Response, Status};
use blst::{min_pk::{PublicKey, AggregatePublicKey, Signature}, BLST_ERROR};
use concilium_shared::{binary, epoch::timestamp_to_epoch_number, ip::ipv4_to_string, BOOTSTRAP_NODES, DST};
use concilium_proto_defs::connection::{
    InitialConnectRequest,
    InitialConnectResponse,
    connection_server::Connection as ConnectionServerSupport,
};
use crate::{
    epoch::client::ClientSupport as EpochClientSupport,
    transaction::client::ClientSupport as TransactionClientSupport,
    connection::client::ClientSupport as ConnectionClientSupport
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
        Self(OriginServer {mempool})
    }
}

#[tonic::async_trait]
impl ConnectionServerSupport for Server {
    async fn initial_connect(&self, request: Request<InitialConnectRequest>) -> Result<Response<InitialConnectResponse>, Status> {
        let request = request.into_inner();

        let new_node_id = request.id;
        let new_node_name = request.name;
        let new_node_public_key = request.public_key.try_into().unwrap();
        let new_node_ip_address = request.ip_address.try_into().unwrap();
        let new_node_port = request.port as u16;
        let new_node_version = request.version;
        let new_node_created_at = request.created_at;
        let new_node_signature: [u8; 96] = request.signature.try_into().unwrap();

        let node_exist = {
            let active_nodes_lock = self.mempool.get_active_nodes();
            let active_nodes = active_nodes_lock.read().await;
            if let None = active_nodes.get_by_public_key(&new_node_public_key) {
                true
            } else {
                false
            }
        };

        if node_exist {
            let self_node_public_key = {
                let self_node_lock = self.mempool.get_self_node();
                let self_node = self_node_lock.read().await; 
                self_node.get_public_key().clone()
            };
    
            let mut available_bootstrap_nodes_public_keys = Vec::new();
            for item in BOOTSTRAP_NODES {
                if hex::encode(self_node_public_key) == item[0] {
                    if let Ok(decoded) = hex::decode(item[0]) {
                        if let Ok(public_key) = PublicKey::from_bytes(&decoded) {
                            available_bootstrap_nodes_public_keys.push(public_key);
                        }
                    }
                } else {
                    if let Ok(decoded) = hex::decode(item[0]) {
                        let node_exist = {
                            let active_nodes_lock = self.mempool.get_active_nodes();
                            let active_nodes = active_nodes_lock.read().await;
                            if let Some(_) = active_nodes.get_by_public_key(decoded.as_slice().try_into().unwrap()) {
                                true
                            } else {
                                false
                            }
                        };

                        if node_exist {
                            if let Ok(public_key) = PublicKey::from_bytes(&decoded) {
                                available_bootstrap_nodes_public_keys.push(public_key);
                            }
                        }
                    }
                }
            }
            let available_bootstrap_nodes_public_keys: Vec<&PublicKey> = available_bootstrap_nodes_public_keys.iter().collect();
    
            let agg_pub = match AggregatePublicKey::aggregate(&available_bootstrap_nodes_public_keys, false) {
                Ok(data) => data.to_public_key(),
                Err(_) => return Ok(initial_connect_false_response())
            };
    
            let agg_sig = match Signature::from_bytes(&new_node_signature) {
                Ok(data) => data,
                Err(_) => return Ok(initial_connect_false_response())
            };
    
            let binary_message = match binary::encode(
                &AwaitingConfirmationNode::new
                (
                    new_node_id, 
                    new_node_name.clone(), 
                    new_node_public_key, 
                    new_node_ip_address, 
                    new_node_port, 
                    new_node_version.clone(), 
                    new_node_created_at
                )
            ) {
                Ok(data) => data,
                Err(_) => return Ok(initial_connect_false_response())
            };

            if agg_sig.verify(false, &binary_message, DST, &[], &agg_pub, true) == BLST_ERROR::BLST_SUCCESS {
                let mempool_clone = Arc::clone(&self.mempool);

                tokio::spawn(async move {
                    sleep(Duration::from_secs(1)).await;
                    
                    let bootstrap_node_signature = {
                        let bootstrap_node_signature_lock = mempool_clone.get_bootstrap_node_signature();
                        let lock = bootstrap_node_signature_lock.lock().await;
                        lock.clone()
                    };
    
                    let node_address = format!("{}:{}", ipv4_to_string(&new_node_ip_address), new_node_port);
                    if let Ok(mut client) = ConnectionClient::connect(&node_address).await {
                        let self_node = {
                            let lock = mempool_clone.get_self_node();
                            let self_node = lock.read().await;
                            self_node.clone()
                        };
                        if let Ok(is_connected) = client.initial_connect(&self_node, &bootstrap_node_signature).await {
                            if is_connected.get_ref().status == true {
                                let epoch_client = EpochClient::connect(&node_address).await.unwrap();                                                      
                                let transaction_client = TransactionClient::connect(&node_address).await.unwrap();                                                      
                                
                                {
                                    let active_nodes_lock = mempool_clone.get_active_nodes();
                                    let mut active_nodes = active_nodes_lock.write().await;
                                    
                                    active_nodes.insert_or_update(
                                        Arc::new(
                                            ActiveNode::new
                                            (
                                                new_node_id, 
                                                new_node_name, 
                                                new_node_public_key, 
                                                new_node_ip_address, 
                                                new_node_port, 
                                                new_node_version, 
                                                new_node_created_at, 
                                                epoch_client,
                                                transaction_client
                                            )
                                        )
                                    );
                                }

                                let node_epoch = timestamp_to_epoch_number(new_node_created_at);
                                let epoch_number = u64::try_from(node_epoch + 5).unwrap();

                                let epoch_pool_read = mempool_clone.get_epoch_pool().get_read();
                                let epoch_pool_write = mempool_clone.get_epoch_pool().get_write();
                                let mut epoch_pool_write_guard = epoch_pool_write.lock().await;

                                if let Some(epoch) = epoch_pool_read.get(&epoch_number) {
                                    if epoch.get_last_node_id() < new_node_id {
                                        epoch_pool_write_guard.update(epoch.get_id().clone(), Arc::new(Epoch::new(epoch.get_id().clone(), new_node_id, epoch.get_final_hash().clone(), epoch.get_random_numbers().to_vec(), epoch.get_hashes().clone())));        
                                    }
                                } else {
                                    epoch_pool_write_guard.insert(epoch_number, Arc::new(Epoch::new(epoch_number, new_node_id, [0; 32], Vec::new(), AHashMap::new())));        
                                };   
                                epoch_pool_write_guard.publish();
                            }
                        } 
                    }    
                });
                return Ok(
                    Response::new(
                        InitialConnectResponse {
                            status: true
                        }
                    )
                );
            } else {
                Ok(initial_connect_false_response())
            }
        } else {
            Ok(
                Response::new(
                    InitialConnectResponse {
                        status: true
                    }
                )
            )
        }
    }
}

fn initial_connect_false_response() -> Response<InitialConnectResponse> {
    Response::new(
        InitialConnectResponse {
            status: false
        }
    )
}
use blst::min_pk::{AggregateSignature, SecretKey, Signature};
use concilium_core::{db::DB, mempool::Mempool, rpc::transaction::Server as OriginServer, transaction::{BroadcastTransactionTemp, Transaction}};
use concilium_core_ext::{mempool::{active_nodes::ActiveNodesSupport, MempoolSupport}, node::{active_node::ActiveNodeSupport, self_node::SelfNodeSupport}, transaction::{broadcast_transaction_temp::BroadcastTransactionTempSupport, transaction::TransactionSupport}};
use concilium_shared::{binary, DST};
use concilium_transaction::{get_accreditation_council_node, get_broadcast_node, put_success_transaction_on_db, validation::{validate_signature_and_txid, validate_utxo_exist_and_values}};
use rayon::prelude::*;
use tonic::{Request, Response, Streaming, Status};
use tokio_stream::{Stream, StreamExt, wrappers::ReceiverStream};
use std::{ops::Deref, pin::Pin, sync::Arc};
use tokio::{sync::mpsc, task::JoinSet};
use concilium_log as log;
use concilium_proto_defs::transaction::{
    transaction_server::Transaction as TransactionServerSupport, AccreditationCouncilRequest, AccreditationCouncilResponse, BroadcastRequest, BroadcastResponse, LeaderRequest, LeaderResponse, SaveRequest, SaveResponse
};
use super::client::ClientSupport;

pub struct Server(pub OriginServer);

impl Deref for Server {
    type Target = OriginServer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait ServerSupport {
    fn new(mempool: Arc<Mempool>, db: Arc<DB>) -> Self;
}

impl ServerSupport for Server {
    fn new(mempool: Arc<Mempool>, db: Arc<DB>) -> Self {
        Self(OriginServer { mempool , db })
    }
}

type LeaderResponseStream = Pin<Box<dyn Stream<Item = Result<LeaderResponse, Status>> + Send>>;
type AccreditationCouncilResponseStream = Pin<Box<dyn Stream<Item = Result<AccreditationCouncilResponse, Status>> + Send>>;
type BroadcastResponseStream = Pin<Box<dyn Stream<Item = Result<BroadcastResponse, Status>> + Send>>;
type SaveResponseStream = Pin<Box<dyn Stream<Item = Result<SaveResponse, Status>> + Send>>;
type ServiceResult<T> = Result<Response<T>, Status>;

#[tonic::async_trait]
impl TransactionServerSupport for Server {
    type LeaderStream = LeaderResponseStream;
    type AccreditationCouncilStream = AccreditationCouncilResponseStream;
    type BroadcastStream = BroadcastResponseStream;
    type SaveStream = SaveResponseStream;

    async fn leader(&self, request: Request<Streaming<LeaderRequest>>) -> ServiceResult<Self::LeaderStream> {
        let (tx, rx) = mpsc::channel(u16::MAX as usize);
        let mut stream = request.into_inner();
        
        let mempool = Arc::clone(&self.mempool);
        let db = Arc::clone(&self.db);
        tokio::spawn(async move {
            'outer: while let Some(request) = stream.next().await {
                match request {
                    Ok(data) => {
                        let transaction = match binary::decode::<Transaction>(&data.transaction) {
                            Ok(t) => t,
                            Err(_) => {
                                tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                                continue;
                            }
                        };

                        if validate_signature_and_txid(&transaction, true) == false {
                            tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                            continue;
                        }

                        let self_node_private_key = {
                            let lock = mempool.get_self_node();
                            let self_node = lock.read().await;

                            self_node.get_private_key().clone()
                        };
                        
                        if validate_utxo_exist_and_values(&transaction, Arc::clone(&mempool)).await == false {
                            tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                            continue;
                        }
                        let transaction_nonce = transaction.get_nonce();

                        let binary_transaction = match binary::encode(&transaction) {
                            Ok(data) => Arc::new(data),
                            Err(_) => {
                                tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                                continue;
                            }
                        };
                        let private_key = match SecretKey::from_bytes(self_node_private_key.as_slice()) {
                            Ok(data) => data,
                            Err(_) => {
                                tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                                continue;
                            }
                        };
                        let binary_transaction_clone = Arc::clone(&binary_transaction);
                        let sign_transaction = private_key.sign(&binary_transaction_clone, DST, &[]);
                        
                        let accreditation_council_node = match get_accreditation_council_node(&transaction, Arc::clone(&mempool)).await {
                            Ok(data) => data,
                            Err(_) => {
                                tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                                continue;
                            }
                        };
                        let mut set = JoinSet::new();
                        
                        for node in accreditation_council_node {
                            let binary = Arc::clone(&binary_transaction);
                            set.spawn(async move {
                                if let Ok(result) = node.get_transaction_client().accreditation_council_request(node.get_id(), transaction_nonce, &binary).await {
                                    if let Ok(response) = result.await {
                                        response
                                    } else {
                                        AccreditationCouncilResponse {
                                            request_id: Vec::new(),
                                            status: false,
                                            signature: Vec::new()
                                        }
                                    }
                                } else {
                                    AccreditationCouncilResponse {
                                        request_id: Vec::new(),
                                        status: false,
                                        signature: Vec::new()
                                    }
                                }
                            });
                        }

                        let mut accreditation_council_signatures: Vec<Signature> = Vec::new();
                        while let Some(result) = set.join_next().await {
                            match result {
                                Ok(response) => {
                                    if response.status == true {
                                        if let Ok(signature) = Signature::from_bytes(&response.signature) {
                                            accreditation_council_signatures.push(signature);
                                        } else {
                                            tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                                            continue 'outer;
                                        }
                                    } else {
                                        tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                                        continue 'outer;
                                    }
                                },
                                Err(_) => {
                                    tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                                    continue 'outer;
                                }
                            }
                            
                        }

                        accreditation_council_signatures.push(sign_transaction);

                        let accreditation_council_signatures = accreditation_council_signatures.par_iter().collect::<Vec<&Signature>>();
                        let accreditation_council_aggregated_signature = match AggregateSignature::aggregate(&accreditation_council_signatures, false) {
                            Ok(data) => data,
                            Err(_) => {
                                tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                                continue;
                            }
                        };
                        
                        let broadcast_transaction_temp = BroadcastTransactionTemp::new(transaction.clone(), accreditation_council_aggregated_signature.to_signature().to_bytes());
                        let binary_broadcast_transaction_temp= match binary::encode(&broadcast_transaction_temp) {
                            Ok(data) => Arc::new(data),
                            Err(_) => {
                                tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                                continue;
                            }
                        };

                        let broadcast_node = match get_broadcast_node(&broadcast_transaction_temp, Arc::clone(&mempool)).await {
                            Ok(data) => data,
                            Err(_) => {
                                tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                                continue;
                            }
                        };
                        let mut set = JoinSet::new();
                        
                        for node in broadcast_node {
                            let binary = Arc::clone(&binary_broadcast_transaction_temp);
                            set.spawn(async move {
                                if let Ok(result) = node.get_transaction_client().broadcast_request(node.get_id(), transaction_nonce, &binary).await {
                                    if let Ok(response) = result.await {
                                        response
                                    } else {
                                        BroadcastResponse {
                                            request_id: Vec::new(),
                                            status: false,
                                            signature: Vec::new(),
                                        }
                                    }
                                } else {
                                    BroadcastResponse {
                                        request_id: Vec::new(),
                                        status: false,
                                        signature: Vec::new(),
                                    }
                                }
                            });
                        }

                        let mut broadcast_signatures: Vec<Signature> = Vec::new();
                        while let Some(result) = set.join_next().await {
                            match result {
                                Ok(response) => {
                                    if response.status == true {
                                        if let Ok(signature) = Signature::from_bytes(&response.signature) {
                                            broadcast_signatures.push(signature);
                                        } else {
                                            tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                                            continue 'outer;
                                        }
                                    } else {
                                        tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                                        continue 'outer;
                                    }
                                },
                                Err(_) => {
                                    tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                                    continue 'outer;
                                }
                            }
                        }

                        let broadcast_signatures = broadcast_signatures.par_iter().collect::<Vec<&Signature>>();
                        let broadcast_aggregated_signature = match AggregateSignature::aggregate(&broadcast_signatures, false) {
                            Ok(data) => data,
                            Err(_) => {
                                tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                                continue;
                            }
                        };

                        if let Err(_) = put_success_transaction_on_db(&transaction, Arc::clone(&mempool), Arc::clone(&db)).await {
                            tx.send(Ok(leader_failde_response(data.request_id.clone()))).await.ok();
                            continue;
                        }

                        
                        let active_nodes = {
                            let active_nodes_lock = mempool.get_active_nodes();
                            let active_nodes = active_nodes_lock.read().await;
                            active_nodes.get_nodes_by_id().clone()
                        };                 
                        for (_, node) in active_nodes {
                            let binary = Arc::clone(&binary_transaction);
                            tokio::spawn(async move {
                                node.get_transaction_client().save_request(node.get_id(), transaction_nonce, &binary).await.ok();
                            });
                        }

                        tx.send(
                            Ok(
                                LeaderResponse {
                                    request_id: data.request_id,
                                    status: true,
                                    accreditation_council_aggregated_signature: accreditation_council_aggregated_signature.to_signature().to_bytes().to_vec(),
                                    broadcast_aggregated_signature: broadcast_aggregated_signature.to_signature().to_bytes().to_vec()
                                }
                            )
                        ).await.ok();
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
            Response::new(Box::pin(out) as Self::LeaderStream)
        )
    }

    async fn accreditation_council(&self, request: Request<Streaming<AccreditationCouncilRequest>>) -> ServiceResult<Self::AccreditationCouncilStream> {
        let (tx, rx) = mpsc::channel(u16::MAX as usize);
        let mut stream = request.into_inner();
        
        let mempool = Arc::clone(&self.mempool);
        tokio::spawn(async move {
            while let Some(request) = stream.next().await {
                match request {
                    Ok(data) => {    
                        let transaction = match binary::decode::<Transaction>(&data.transaction) {
                            Ok(t) => t,
                            Err(_) => {
                                tx.send(Ok(accreditation_council_failde_response(data.request_id.clone()))).await.ok();
                                continue;
                            }
                        };
                        
                        if validate_signature_and_txid(&transaction, true) == false {
                            tx.send(Ok(accreditation_council_failde_response(data.request_id.clone()))).await.ok();
                            continue;
                        }

                        let self_node_private_key = {
                            let lock = mempool.get_self_node();
                            let self_node = lock.read().await;

                            self_node.get_private_key().clone()
                        };
                        
                        if validate_utxo_exist_and_values(&transaction, Arc::clone(&mempool)).await == false {
                            tx.send(Ok(accreditation_council_failde_response(data.request_id.clone()))).await.ok();
                            continue;
                        }

                        let binary_transaction = match binary::encode(&transaction) {
                            Ok(data) => data,
                            Err(_) =>  {
                                tx.send(Ok(accreditation_council_failde_response(data.request_id.clone()))).await.ok();
                                continue;
                            }
                        };
                        let private_key = match SecretKey::from_bytes(self_node_private_key.as_slice()) {
                            Ok(data) => data,
                            Err(_) => {
                                tx.send(Ok(accreditation_council_failde_response(data.request_id.clone()))).await.ok();
                                continue;
                            }
                        };
                        let sign_transaction = private_key.sign(&binary_transaction, DST, &[]);
                    
                        tx.send(
                            Ok(
                                AccreditationCouncilResponse {
                                    request_id: data.request_id,
                                    status: true,
                                    signature: sign_transaction.to_bytes().to_vec()
                                }
                            )
                        ).await.ok();
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
            Response::new(Box::pin(out) as Self::AccreditationCouncilStream)
        )
    }
    
    async fn broadcast(&self, request: Request<Streaming<BroadcastRequest>>) -> ServiceResult<Self::BroadcastStream> {
        let (tx, rx) = mpsc::channel(u16::MAX as usize);
        let mut stream = request.into_inner();
        
        let mempool = Arc::clone(&self.mempool);
        tokio::spawn(async move {
            while let Some(request) = stream.next().await {
                match request {
                    Ok(data) => {    
                        let self_node_private_key = {
                            let lock = mempool.get_self_node();
                            let self_node = lock.read().await;

                            self_node.get_private_key().clone()
                        };
                                                
                        let private_key = match SecretKey::from_bytes(self_node_private_key.as_slice()) {
                            Ok(data) => data,
                            Err(_) => {
                                tx.send(Ok(broadcast_failde_response(data.request_id.clone()))).await.ok();
                                continue;
                            }
                        };
                        let sign = private_key.sign(&data.broadcast_transaction_temp, DST, &[]);
                    
                        tx.send(
                            Ok(
                                BroadcastResponse {
                                    request_id: data.request_id,
                                    status: true,
                                    signature: sign.to_bytes().to_vec()
                                }
                            )
                        ).await.ok();
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
            Response::new(Box::pin(out) as Self::BroadcastStream)
        )
    }
    
    async fn save(&self, request: Request<Streaming<SaveRequest>>) -> ServiceResult<Self::SaveStream> {
        let (tx, rx) = mpsc::channel(u16::MAX as usize);
        let mut stream = request.into_inner();
        
        let mempool = Arc::clone(&self.mempool);
        let db = Arc::clone(&self.db);
        tokio::spawn(async move {
            while let Some(request) = stream.next().await {
                match request {
                    Ok(data) => {    
                        let transaction = match binary::decode::<Transaction>(&data.transaction) {
                            Ok(t) => t,
                            Err(_) => {
                                tx.send(Ok(save_failde_response(data.request_id.clone()))).await.ok();
                                continue;
                            }
                        };

                        if let Err(_) = put_success_transaction_on_db(&transaction, Arc::clone(&mempool), Arc::clone(&db)).await {
                            tx.send(Ok(save_failde_response(data.request_id.clone()))).await.ok();
                            continue;
                        }

                        tx.send(
                            Ok(
                                SaveResponse {
                                    request_id: data.request_id,
                                    status: true,
                                }
                            )
                        ).await.ok();
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
            Response::new(Box::pin(out) as Self::SaveStream)
        )
    }
}

fn leader_failde_response(request_id: Vec<u8>) -> LeaderResponse {
    LeaderResponse {
        request_id: request_id,
        status: false,
        accreditation_council_aggregated_signature: Vec::new(),
        broadcast_aggregated_signature: Vec::new()
    }
}

fn accreditation_council_failde_response(request_id: Vec<u8>) -> AccreditationCouncilResponse {
    AccreditationCouncilResponse {
        request_id: request_id,
        status: false,
        signature: Vec::new()
    }
}

fn broadcast_failde_response(request_id: Vec<u8>) -> BroadcastResponse {
    BroadcastResponse {
        request_id: request_id,
        status: false,
        signature: Vec::new()
    }
}

fn save_failde_response(request_id: Vec<u8>) -> SaveResponse {
    SaveResponse {
        request_id: request_id,
        status: false,
    }
}
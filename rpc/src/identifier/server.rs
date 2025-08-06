use std::{ops::Deref, sync::Arc};
use blst::{min_pk::{AggregatePublicKey, PublicKey, SecretKey, Signature}, BLST_ERROR};
use concilium_core::{mempool::Mempool, node::{AwaitingConfirmationNode, SerializableNode}, rpc::identifier::Server as OriginServer};
use concilium_core_ext::{mempool::{active_nodes::ActiveNodesSupport, nodes_awaiting_confirmation::NodesAwaitingConfirmationSupport, MempoolSupport}, node::{awaiting_confirmation_node::AwaitingConfirmationNodeSupport, self_node::SelfNodeSupport, serializable_node::SerializableNodeSupport}};
use concilium_shared::{binary, BOOTSTRAP_NODES, DST};
use tonic::{Request, Response, Status};
use rayon::prelude::*;
use concilium_proto_defs::identifier::{
    GetIdRequest,
    GetIdResponse,
    ValidateIdRequest,
    ValidateIdResponse,
    identifier_server::Identifier as IdentifierServerSupport
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
impl IdentifierServerSupport for Server {
    async fn get_id(&self, request: Request<GetIdRequest>) -> Result<Response<GetIdResponse>, Status> {
        let new_node = request.into_inner();
        
        let new_node_name = new_node.name;
        let new_node_public_key = new_node.public_key.try_into().unwrap();
        let new_node_ip_address = new_node.ip_address.try_into().unwrap();
        let new_node_port = new_node.port as u16;
        let new_node_version = new_node.version;
        let new_node_created_at = new_node.created_at;

        let nodes_awaiting_confirmation = self.mempool.get_nodes_awaiting_confirmation();
        let mut nodes_awaiting_confirmation = nodes_awaiting_confirmation.write().await;
        
        let last_node = if nodes_awaiting_confirmation.len() > 0 {
            if let Some(node) = nodes_awaiting_confirmation.get(&new_node_public_key) {
                node.get_id()
            } else {
                match nodes_awaiting_confirmation.get_last_id() {
                    Some(id) => id + 1,
                    None => 2
                }
            }
        } else {
            let active_nodes = self.mempool.get_active_nodes();
            let active_nodes = active_nodes.read().await;
            
            match active_nodes.get_last_id() {
                Some(id) => id + 1,
                None => 2
            }
        };

        nodes_awaiting_confirmation.insert_or_update(
            AwaitingConfirmationNode::new(
                last_node,
                new_node_name.clone(),
                new_node_public_key,
                new_node_ip_address,
                new_node_port,
                new_node_version.clone(),
                new_node_created_at
            )
        );

        let self_node_private_key = {
            let self_node = self.mempool.get_self_node();
            let self_node = self_node.read().await;
            self_node.get_private_key().clone()
        };

        if let Ok(private_key) = SecretKey::from_bytes(&self_node_private_key) {
            if let Ok(message) = binary::encode(
                &SerializableNode::new
                (
                    last_node, 
                    new_node_name,
                    new_node_public_key,
                    new_node_ip_address,
                    new_node_port,
                    new_node_version,
                    new_node_created_at
                )
            ) {
                let signature = private_key.sign(&message, DST, &[]).to_bytes();
                
                return Ok(Response::new(GetIdResponse {
                    status: true,
                    signature: signature.to_vec(),
                    id: last_node,
                }));
            }
        } 
        
        return Ok(get_id_false_response());
    }

    async fn validate_id(&self, request: Request<ValidateIdRequest>) -> Result<Response<ValidateIdResponse>, Status> {
        let request = request.get_ref();

        let message = match binary::decode::<SerializableNode>(&request.message) {
            Ok(message) => message,
            Err(_) => return Ok(validate_id_false_response())
        };

        let signature: [u8; 96] = request.signatures.clone().try_into().unwrap();
        
        let nodes_awaiting_confirmation_lock = self.mempool.get_nodes_awaiting_confirmation();
        let nodes_awaiting_confirmation = nodes_awaiting_confirmation_lock.read().await;

        if let None = nodes_awaiting_confirmation.get(&message.get_public_key()) {
            return Ok(validate_id_false_response());
        }

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
                    let decoded: [u8; 48] = decoded.try_into().unwrap();

                    let active_nodes_lock = self.mempool.get_active_nodes();
                    let active_nodes = active_nodes_lock.read().await;
                    if let Some(_) = active_nodes.get_by_public_key(&decoded) {
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
            Err(_) => return Ok(validate_id_false_response())
        };

        let agg_sig = match Signature::from_bytes(&signature) {
            Ok(data) => data,
            Err(_) => return Ok(validate_id_false_response())
        };

        let binary_message = match binary::encode(&message) {
            Ok(data) => data,
            Err(_) => return Ok(validate_id_false_response())
        };
        
        if agg_sig.verify(false, &binary_message, DST, &[], &agg_pub, true) == BLST_ERROR::BLST_SUCCESS {
            let nodes = {
                let active_nodes_lock = self.mempool.get_active_nodes();
                let active_nodes = active_nodes_lock.read().await;
                active_nodes.get_nodes_by_public_key().clone()
            };

            let mut nodes: Vec<SerializableNode> = nodes.par_iter().map(|(_, node)| {
                SerializableNode::new(
                    node.id, 
                    node.name.clone(), 
                    node.public_key, 
                    node.ip_address, 
                    node.port, 
                    node.version.clone(), 
                    node.created_at
                )
            }).collect();

            {
                let self_node_lock = self.mempool.get_self_node();
                let self_node = self_node_lock.read().await;

                nodes.push(SerializableNode::new(
                    self_node.get_id(), 
                    self_node.get_name().to_vec(), 
                    *self_node.get_public_key(), 
                    *self_node.get_ip_address(), 
                    self_node.get_port(), 
                    self_node.get_version().to_vec(), 
                    self_node.get_created_at()
                ));
            }

            

            if let Ok(data) = binary::encode(&nodes){
                return Ok(Response::new(ValidateIdResponse { status : true, nodes: data }));
            } else {
                Ok(validate_id_false_response())
            }
        } else {
            Ok(validate_id_false_response())
        }
    }
}

fn get_id_false_response() -> Response<GetIdResponse> {
    Response::new(GetIdResponse{
        status: false,
        signature: b"None".to_vec(),
        id: 0,
    })
}

fn validate_id_false_response() -> Response<ValidateIdResponse> {
    Response::new(ValidateIdResponse {
        status: false,
        nodes: b"None".to_vec(),
    })
}
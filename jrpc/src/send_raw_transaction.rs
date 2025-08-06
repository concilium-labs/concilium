use std::sync::Arc;
use blst::min_pk::{AggregateSignature, SecretKey, Signature};
use concilium_core::{db::DB, jrpc::transaction::{SendRawTransactionRequest, SendRawTransactionResponse}, mempool::Mempool, transaction::BroadcastTransactionTemp};
use concilium_core_ext::{mempool::{active_nodes::ActiveNodesSupport, MempoolSupport}, node::{active_node::ActiveNodeSupport, self_node::SelfNodeSupport}, transaction::{broadcast_transaction_temp::BroadcastTransactionTempSupport, send_raw_transaction_response::SendRawTransactionResponseSupport, transaction::TransactionSupport}};
use concilium_proto_defs::transaction::{AccreditationCouncilResponse, BroadcastResponse};
use concilium_shared::{binary, DST};
use concilium_transaction::{get_accreditation_council_node, get_broadcast_node, get_leader, put_success_transaction_on_db, send_raw_transaction_request_to_transaction, validation::{validate_signature_and_txid, validate_utxo_exist_and_values}};
use jsonrpsee::types::{ErrorObject, Params};
use concilium_rpc::transaction::client::ClientSupport;
use tokio::task::JoinSet;
use rayon::prelude::*;

pub async fn handler(params: Params<'_>, mempool: Arc<Mempool>, db: Arc<DB>) -> Result<SendRawTransactionResponse, ErrorObject<'static>> {
    let trx_request: SendRawTransactionRequest = match params.parse() {
        Ok(t) => t,
        Err(_) => return Err(error_response(-32602, "invalid params(convert to SendRawTransactionRequest is failed)"))
    };
    
    let transaction = match send_raw_transaction_request_to_transaction(trx_request) {
        Ok(t) => t,
        Err(_) => return Err(error_response(-32602, "invalid params(convert to Transaction is failed)"))
    };
    
    if validate_signature_and_txid(&transaction, false) == false {
        return Err(error_response(-32602, "invalid signature"));
    }
    
    if let Ok(result) = get_leader(&transaction, Arc::clone(&mempool)).await {
        if let Some(node) = result {
            let transaction_binary = match binary::encode(&transaction) {
                Ok(data) => data,
                Err(_) => return Err(error_response(-32602, "transaction to binary is failed - leader"))
            };
            
            let response = match node.transaction_client.leader_request(
                node.get_id(),
                transaction.get_nonce(),
                &transaction_binary
            ).await {
                Ok(data) => data,
                Err(_) => return Err(error_response(-32602, "send to leader request is failed"))
            };

            match response.await {
                Ok(data) => {
                    if data.status == true {
                        return Ok(
                            successful_transaction(
                                transaction.get_txid().clone(), 
                                data.accreditation_council_aggregated_signature.try_into().unwrap_or([0; 96]), 
                                data.broadcast_aggregated_signature.try_into().unwrap_or([0; 96])
                            ).await
                        );
                    }
                    
                    return Err(error_response(-32602, "transaction is failed"))
                },
                Err(_) => {
                    return Err(error_response(-32602, "transaction is failed(request error)"))
                }
            }
        }
    } else {
        return Err(error_response(-32602, "find leader is failed"))
    }
    
    let self_node_private_key = {
        let lock = mempool.get_self_node();
        let self_node = lock.read().await;

        self_node.get_private_key().clone()
    };
    
    if validate_utxo_exist_and_values(&transaction, Arc::clone(&mempool)).await == false {        
        return Err(error_response(-32602, "utxo is invaild"))
    }
    
    let transaction_nonce = transaction.get_nonce();

    let binary_transaction = match binary::encode(&transaction) {
        Ok(data) => Arc::new(data),
        Err(_) => return Err(error_response(-32602, "transaction to binary is failed"))
    };
    let private_key = match SecretKey::from_bytes(self_node_private_key.as_slice()) {
        Ok(data) => data,
        Err(_) => return Err(error_response(-32602, "invalid private key"))
    };
    let binary_transaction_clone = Arc::clone(&binary_transaction);
    let sign_transaction = private_key.sign(&binary_transaction_clone, DST, &[]);

    let accreditation_council_node = match get_accreditation_council_node(&transaction, Arc::clone(&mempool)).await {
        Ok(data) => data,
        Err(_) => return Err(error_response(-32602, "send to accreditation council request is failed"))
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
    while let Some(data) = set.join_next().await {
        match data {
            Ok(response ) => {
                if response.status == true {
                    if let Ok(signature) = Signature::from_bytes(&response.signature) {
                        accreditation_council_signatures.push(signature);
                    } else {
                        return Err(error_response(-32602, "accreditation council node can not loaded signature"));
                    }
                } else {
                    return Err(error_response(-32602, "accreditation council node not signed transaction"))
                }
            },
            Err(_) => return Err(error_response(-32602, "send for accreditation council node to signed is failed"))
        }
        
    }

    accreditation_council_signatures.push(sign_transaction);

    let accreditation_council_signatures = accreditation_council_signatures.par_iter().collect::<Vec<&Signature>>();
    let accreditation_council_aggregated_signature = match AggregateSignature::aggregate(&accreditation_council_signatures, false) {
        Ok(data) => data,
        Err(_) => return Err(error_response(-32602, "accreditation council aggregate signature is failed"))
    };
    
    let broadcast_transaction_temp = BroadcastTransactionTemp::new(transaction.clone(), accreditation_council_aggregated_signature.to_signature().to_bytes());
    let binary_broadcast_transaction_temp= match binary::encode(&broadcast_transaction_temp) {
        Ok(data) => Arc::new(data),
        Err(_) => return Err(error_response(-32602, "broadcast transaction temp to binary is failed"))
    };

    let broadcast_node = match get_broadcast_node(&broadcast_transaction_temp, Arc::clone(&mempool)).await {
        Ok(data) => data,
        Err(_) => return Err(error_response(-32602, "send to broadcast node request is failed"))
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
    while let Some(data) = set.join_next().await {
        match data {
            Ok(response ) => {
                if response.status == true {
                    if let Ok(signature) = Signature::from_bytes(&response.signature) {
                        broadcast_signatures.push(signature);
                    } else {
                        return Err(error_response(-32602, "broadcast node can not loaded signature"));
                    }
                } else {
                    return Err(error_response(-32602, "broadcast node not signed transaction "))
                }
            },
            Err(_) => return Err(error_response(-32602, "send for broadcast node to signed is failed"))
        }
    }

    let broadcast_signatures = broadcast_signatures.par_iter().collect::<Vec<&Signature>>();
    let broadcast_aggregated_signature = match AggregateSignature::aggregate(&broadcast_signatures, false) {
        Ok(data) => data,
        Err(_) => return Err(error_response(-32602, "broadcast aggregate signature is failed"))
    };

    if let Err(_) = put_success_transaction_on_db(&transaction, Arc::clone(&mempool), Arc::clone(&db)).await {        
        return Err(error_response(-32602, "internal error(save on db)"));
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

    return Ok(successful_transaction(
        transaction.get_txid().clone(), 
        accreditation_council_aggregated_signature.to_signature().to_bytes(), 
        broadcast_aggregated_signature.to_signature().to_bytes()).await
    );
}

fn error_response(code: i32, message: &str) -> ErrorObject {
    ErrorObject::owned(
        code,
        message,
        None::<String>,
    )
}

async fn successful_transaction(txid: [u8; 32], accreditation_council_aggregated_signature: [u8; 96], broadcast_aggregated_signature: [u8; 96]) -> SendRawTransactionResponse {
    SendRawTransactionResponse::new(
        true, 
        hex::encode(txid), 
        hex::encode(accreditation_council_aggregated_signature), 
        hex::encode(broadcast_aggregated_signature), 
    )
}
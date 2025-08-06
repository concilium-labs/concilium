use std::sync::Arc;
use ahash::{AHashMap, AHashSet};
use concilium_core::{db::DB, jrpc::transaction::SendRawTransactionRequest, mempool::Mempool, node::ActiveNode, transaction::{BroadcastTransactionTemp, TXInput, TXOutput, Transaction}};
use concilium_core_ext::{
    chain_state::ChainStateSupport, db::DBSupport, epoch::{EpochPoolSupport, EpochSupport}, mempool::{active_nodes::ActiveNodesSupport, MempoolSupport}, node::active_node::ActiveNodeSupport, temporary_node_ids::TemporaryNodeIdsSupport, transaction::{broadcast_transaction_temp::BroadcastTransactionTempSupport, transaction::TransactionSupport, txinput::TXInputSupport, txoutput::TXOutputSupport}
};
use concilium_error::Error;
use concilium_shared::{binary, chacha20::generate_random_number_by_seed, coventor::vec::unsigned_int::vec_to_unsigned_int, epoch::timestamp_to_epoch_number, sha::sha256, transaction::calculating_nnr};

pub mod validation;

pub fn send_raw_transaction_request_to_transaction(trx_request: SendRawTransactionRequest) -> Result<Transaction, Error> {
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    for item in trx_request.vin {
        inputs.push(TXInput::new(
            hex::decode(item.txid)?.try_into()?, 
            item.vout, 
        ));
    }
    
    for item in trx_request.vout {
        outputs.push(TXOutput::new(
            item.value, 
            hex::decode(item.public_key)?.try_into()?
        ));
    }

    let mut transaction = Transaction::new(
        [0; 32], 
        hex::decode(trx_request.from)?.try_into()?, 
        [0; 64],
        trx_request.nonce, 
        trx_request.created_at, 
        inputs, 
        outputs
    );

    let binary_transaction = binary::encode(&transaction)?;
    let txid = sha256(&binary_transaction);

    transaction.set_txid(txid);
    transaction.set_signature(hex::decode(trx_request.signature)?.try_into()?);
    
    Ok(transaction)
}

pub async fn get_leader(transaction: &Transaction, mempool: Arc<Mempool>) -> Result<Option<Arc<ActiveNode>>, Error> {
    let transaction_epoch_number = timestamp_to_epoch_number(transaction.get_created_at()) as u64;
    let hash = sha256(transaction.get_from());
   
    let epoch_pool = mempool.get_epoch_pool().get_read();
    let last_node_id = match epoch_pool.get(&transaction_epoch_number) {
        Some(data) => data.get_last_node_id(),
        None => return Err(Error::new("epoch not found"))
    };

    let id = generate_random_number_by_seed(hash, last_node_id, 1)[0];

    let temporary_node_ids = mempool.get_temporary_node_ids().get_read();
    let leader_id = match temporary_node_ids.get(&transaction_epoch_number) {
        Some(m) => {
            match m.get(&id) {
                Some(data) => data.clone(),
                None => return Err(Error::new("epoch not found"))
            }
        },
        None => return Err(Error::new("epoch not found"))
    };

    let lock = mempool.get_active_nodes();
    let nodes = lock.read().await;
    
    if let Some(n) = nodes.get_by_id(leader_id) {
        Ok(Some(Arc::clone(&n)))
    } else {
        Ok(None)
    }
}

pub async fn get_accreditation_council_node(transaction: &Transaction, mempool: Arc<Mempool>) -> Result<Vec<Arc<ActiveNode>>, Error> {
    let (_, node_count_per_before_cycle, node_count_per_current_cycle, node_count_per_trx) = get_nmac(transaction, Arc::clone(&mempool)).await;
    let transaction_epoch_number = timestamp_to_epoch_number(transaction.get_created_at()) as u64;
    let mut exist_self_on_consensus = false;
    let epoch_pool = mempool.get_epoch_pool().get_read();
    
    let last_node_id = match epoch_pool.get(&transaction_epoch_number) {
        Some(data) => data.get_last_node_id(),
        None => return Err(Error::new("epoch not found"))
    };

    let temporary_node_ids = mempool.get_temporary_node_ids().get_read();
    
    let active_nodes_lock = mempool.get_active_nodes();
    let active_nodes = active_nodes_lock.read().await;
    
    let public_key_hash = sha256(&binary::encode(transaction.get_from())?);
    let transaction_hash = sha256(&binary::encode(&transaction)?);

    let before_temporary_node_ids = match temporary_node_ids.get(&(transaction_epoch_number - 1)) {
        Some(data) => data,
        None => return Err(Error::new("temporary node ids not found"))
    };
    let current_temporary_node_ids = match temporary_node_ids.get(&transaction_epoch_number) {
        Some(data) => data,
        None => return Err(Error::new("temporary node ids not found"))
    }; 

    let mut nodes = AHashMap::new();

    let ids_by_current_cycle = generate_random_number_by_seed(public_key_hash, last_node_id, node_count_per_current_cycle);
    for item in ids_by_current_cycle {
        let node_id = match current_temporary_node_ids.get(&item) {
            Some(data) => data.clone(),
            None => return Err(Error::new("node_id not found"))
        };

        if let Some(n) = active_nodes.get_by_id(node_id) {
            nodes.insert(n.get_id(), Arc::clone(&n));
        } else {
            exist_self_on_consensus = true;
        }
    }

    let blacklist: AHashSet<u32> = nodes.keys().cloned().collect();

    let nodes_needed_for_before_cycle = node_count_per_before_cycle as usize;
    let mut initial_before_cycle_how_many= node_count_per_before_cycle;
    loop {
        let ids_by_before_cycle = generate_random_number_by_seed(public_key_hash, last_node_id, initial_before_cycle_how_many);
        let mut nodes_by_before_cycle = AHashMap::new();
        
        for item in ids_by_before_cycle {
            let node_id = match before_temporary_node_ids.get(&item) {
                Some(data) => data.clone(),
                None => return Err(Error::new("node_id not found"))
            };
            
            if let Some(n) = active_nodes.get_by_id(node_id) {
                if !blacklist.contains(&n.get_id()) {
                    nodes_by_before_cycle.insert(n.get_id(), Arc::clone(n));
                }
            } else {
                exist_self_on_consensus = true;
            }
        }

        if nodes_by_before_cycle.len() != nodes_needed_for_before_cycle {
            initial_before_cycle_how_many += 1;
        } else {
            nodes.extend(nodes_by_before_cycle);
            break;
        }
    }
    
    let blacklist: AHashSet<u32> = nodes.keys().cloned().collect();

    let mut nodes_needed_for_transaction = node_count_per_trx as usize;
    if exist_self_on_consensus {
        nodes_needed_for_transaction -= 1;
    } 

    let mut initial_transaction_how_many = nodes_needed_for_transaction;
    loop {
        let ids_by_transaction = generate_random_number_by_seed(transaction_hash, last_node_id, initial_transaction_how_many as u32);        
        let mut nodes_by_transaction = AHashMap::new();
        
        for item in ids_by_transaction {
            let node_id = match current_temporary_node_ids.get(&item) {
                Some(data) => data.clone(),
                None => return Err(Error::new("node_id not found"))
            };
        
            if let Some(n) = active_nodes.get_by_id(node_id) {
                if !blacklist.contains(&n.get_id()) {
                    nodes_by_transaction.insert(n.get_id(), Arc::clone(n));
                }
            }
        }

        if nodes_by_transaction.len() == nodes_needed_for_transaction {
            nodes.extend(nodes_by_transaction);
            break;
        } else {            
            initial_transaction_how_many += 1;
        }
    }

    Ok(nodes.into_values().collect())
}

pub async fn get_broadcast_node(broadcast_transaction_temp: &BroadcastTransactionTemp, mempool: Arc<Mempool>) -> Result<Vec<Arc<ActiveNode>>, Error> {
    let (nnr, _, _, _) = get_nmac(broadcast_transaction_temp.get_transaction(), Arc::clone(&mempool)).await;
    let nnr = (nnr as f32) * 0.10;
    let mut nnr = nnr.ceil() as u32;

    if nnr > 128 {
        nnr = 128;
    }

    let transaction_epoch_number = timestamp_to_epoch_number(broadcast_transaction_temp.get_transaction().get_created_at()) as u64;

    let epoch_pool = mempool.get_epoch_pool().get_read();
    let last_node_id = match epoch_pool.get(&transaction_epoch_number) {
        Some(data) => data.get_last_node_id(),
        None => return Err(Error::new("epoch not found"))
    };

    let temporary_node_ids = mempool.get_temporary_node_ids().get_read();
    
    let active_nodes_lock = mempool.get_active_nodes();
    let active_nodes = active_nodes_lock.read().await;
    
    let broadcast_transaction_temp_hash = sha256(&binary::encode(broadcast_transaction_temp)?);

    let current_temporary_node_ids = match temporary_node_ids.get(&transaction_epoch_number) {
        Some(data) => data,
        None => return Err(Error::new("node_id not found"))
    }; 

    let mut nodes = AHashMap::new();

    let nodes_needed = nnr as usize;
    let mut initial_how_many = nnr;
    loop {
        let ids = generate_random_number_by_seed(broadcast_transaction_temp_hash, last_node_id, initial_how_many);
        let mut nodes_broadcast = AHashMap::new();
        
        for item in ids {
            let node_id = match current_temporary_node_ids.get(&item) {
                Some(data) => data.clone(),
                None => return Err(Error::new("node_id not found"))
            };

            if let Some(n) = active_nodes.get_by_id(node_id) {
                nodes_broadcast.insert(n.get_id(), Arc::clone(n));
            }
        }

        if nodes_broadcast.len() != nodes_needed {
            initial_how_many += 1;
        } else {
            nodes.extend(nodes_broadcast);
            break;
        }
    }

    Ok(nodes.into_values().collect())
}

/* 
    Number of members of the Accreditation Council

    returns (
        all nodes count, 
        before cycle nodes count, 
        current cycle nodes count, 
        nodes count by transaction
    )
*/
pub async fn get_nmac(transaction: &Transaction, mempool: Arc<Mempool>) -> (u32, u32, u32, u32) {
    let sum_vouts: f32 = transaction.get_vout()
    .iter()
    .map(|v| { v.value })
    .sum();

    let mut nnr = calculating_nnr(sum_vouts);
    if nnr < 128 {
        nnr = 128;
    }

    let node_count = {
        let lock = mempool.get_active_nodes();
        let nodes = lock.read().await;
        (nodes.get_nodes_by_id().len() + 1) as u32
    };

    if nnr > node_count {
        nnr = node_count;
    }
    
    let node_count_per_section = nnr / 3;
    let node_count_per_before_cycle = node_count_per_section;
    let node_count_per_current_cycle = node_count_per_section;
    let node_count_per_trx = nnr - (node_count_per_before_cycle + node_count_per_current_cycle);

    (nnr, node_count_per_before_cycle, node_count_per_current_cycle, node_count_per_trx)
}

pub async fn put_success_transaction_on_db(transaction: &Transaction, mempool: Arc<Mempool>, db: Arc<DB>) -> Result<(), Error> {
    let db_last_transaction_id = db.get("last_transaction_id")?;
    
    if let Some(id) = db_last_transaction_id {
        let u64_id = vec_to_unsigned_int::<u64>(&id);
        
        if let Some(last_transaction_id) = u64_id {
            let txid_hex = hex::encode(transaction.get_txid().clone());
        
            let binary_transaction = binary::encode(transaction)?;

            db.put(format!("transaction.{}", txid_hex).as_str(),&binary_transaction)?;
            
            db.put(format!("transaction.id.{}", (last_transaction_id + 1)).as_str(),&txid_hex)?;
            db.put("last_transaction_id", &(last_transaction_id + 1).to_le_bytes())?;

            let lock = mempool.get_utxos();
            let mut utxos = lock.write().await;

            for input in transaction.get_vin() {
                utxos.remove(&(input.get_txid().clone(), input.get_vout()));    
            }

            let chain_state_lock = mempool.get_chain_state();
            let mut chain_state = chain_state_lock.write().await;

            let mut outputs_fee = 0.0;
            for (index, output ) in transaction.get_vout().iter().enumerate() {
                outputs_fee += output.get_value();
                
                let account_balance = match chain_state.get_balances().get(output.get_public_key()) {
                    Some(data) => *data + output.value,
                    None => output.value
                };
                chain_state.get_mut_balances().insert(output.get_public_key().clone(), account_balance);

                if let Some(data) = chain_state.get_mut_transactions().get_mut(output.get_public_key()) {
                    data.insert(transaction.get_txid().clone());
                } else {
                    let mut hash_set: AHashSet<[u8; 32]> = AHashSet::new();
                    hash_set.insert(transaction.get_txid().clone());
                    
                    chain_state.get_mut_transactions().insert(output.get_public_key().clone(), hash_set);
                }
                utxos.insert((transaction.get_txid().clone(), index), output.clone());
            }

            let account_balance = match chain_state.get_balances().get(transaction.get_from()) {
                Some(data) => *data - outputs_fee,
                None => -outputs_fee
            };
            chain_state.get_mut_balances().insert(transaction.get_from().clone(), account_balance);

            if let Some(data) = chain_state.get_mut_transactions().get_mut(transaction.get_from()) {
                data.insert(transaction.get_txid().clone());
            } else {
                let mut hash_set = AHashSet::new();
                hash_set.insert(transaction.get_txid().clone());

                chain_state.get_mut_transactions().insert(transaction.get_from().clone(), hash_set);
            }

            Ok(())
        } else {
            Err(Error::new("last transaction id not found"))
        }
    } else {
        Err(Error::new("last transaction id not found"))
    }
}
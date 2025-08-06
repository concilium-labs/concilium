use std::sync::Arc;
use ahash::AHashSet;
use concilium_core::{db::DB, mempool::Mempool, transaction::{TXOutput, Transaction}};
use concilium_core_ext::{chain_state::ChainStateSupport, db::DBSupport, mempool::MempoolSupport, transaction::{transaction::TransactionSupport, txinput::TXInputSupport, txoutput::TXOutputSupport}};
use concilium_shared::{binary, coventor::vec::unsigned_int::vec_to_unsigned_int};
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Serialize, Deserialize, Debug)]
struct GenesisTransaction {
    public_key: String,
    private_key: String,
    transaction: GTransaction
}

#[derive(Serialize, Deserialize, Debug)]
struct GTransaction {
    txid: String,
    from: String, // public key
    signature: String,
    nonce: u64,
    created_at: i64,
    vin: Vec<GTXInput>,
    vout: Vec<GTXOutput>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GTXInput {
    txid: String,
    vout: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct GTXOutput {
    value: f32,
    public_key: String
}

pub async fn load_genesis_transactions(mempool: Arc<Mempool>, db: Arc<DB>,) {
    let mut transactions_file = File::open("genesis_transactions.json").await.unwrap();

    let mut buffer = String::new();

    transactions_file.read_to_string(&mut buffer).await.unwrap();

    let genesis_transactions: Vec<GenesisTransaction> = serde_json::from_str(&buffer).unwrap();

    let utxos_lock = mempool.get_utxos();
    let mut utxos = utxos_lock.write().await;

    let chain_state_lock = mempool.get_chain_state();
    let mut chain_state = chain_state_lock.write().await;

    for trx in genesis_transactions {
        let last_transaction_id = db.get("last_transaction_id").unwrap();
        let last_transaction_id : u64 = if let Some(data) = last_transaction_id {
            vec_to_unsigned_int::<u64>(&data).unwrap()
        } else {
            0
        };

        let public_key: [u8; 32] = hex::decode(trx.public_key).unwrap().try_into().unwrap();

        let transaction = Transaction::new(
            hex::decode(trx.transaction.txid.clone()).unwrap().try_into().unwrap(), 
            [0; 32],
            [0; 64],
            trx.transaction.nonce,
            trx.transaction.created_at,
            Vec::new(), 
            vec![
                TXOutput::new(
                    trx.transaction.vout[0].value, 
                    public_key.clone()
                )
            ],
        );

        let binary_transaction = binary::encode(&transaction).unwrap();
        db.put(format!("transaction.{}", trx.transaction.txid.clone()).as_str(),&binary_transaction).unwrap();
        
        db.put(format!("transaction.id.{}", (last_transaction_id + 1)).as_str(),&trx.transaction.txid).unwrap();
        db.put("last_transaction_id", &(last_transaction_id + 1).to_le_bytes()).unwrap();

        utxos.insert((transaction.txid.clone(), 0), transaction.vout[0].clone());

        chain_state.get_mut_balances().insert(public_key.clone(), trx.transaction.vout[0].value);

        let mut transaction_hash_set = AHashSet::new();
        transaction_hash_set.insert(transaction.txid);
        chain_state.get_mut_transactions().insert(public_key, transaction_hash_set);
    }

    db.put("included_genesis_transactions", b"true").unwrap();  
}

pub async fn load_transactions(mempool: Arc<Mempool>, db: Arc<DB>) {
    let last_transaction_id = vec_to_unsigned_int::<u64>(&db.get("last_transaction_id").unwrap().unwrap()).unwrap();

    let utxos_lock = mempool.get_utxos();
    let mut utxos = utxos_lock.write().await;

    let chain_state_lock = mempool.get_chain_state();
    let mut chain_state = chain_state_lock.write().await;

    for id in 1..=last_transaction_id {
        let txid = db.get(format!("transaction.id.{}", id).as_str()).unwrap().unwrap();
        let txid = String::from_utf8(txid).unwrap();

        let transaction = db.get(format!("transaction.{}", txid).as_str()).unwrap().unwrap();
        let transaction = binary::decode::<Transaction>(&transaction).unwrap();

        for input in transaction.get_vin() {
            utxos.remove(&(*input.get_txid(), input.get_vout()));    
        }

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

            utxos.insert((*transaction.get_txid(), index), *output);
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
    }
}
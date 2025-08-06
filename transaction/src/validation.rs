use std::sync::Arc;
use concilium_core::{mempool::Mempool, transaction::Transaction};
use concilium_core_ext::{mempool::MempoolSupport, transaction::{transaction::TransactionSupport, txinput::TXInputSupport, txoutput::TXOutputSupport}};
use concilium_shared::{binary, sha::sha256};
use ed25519_dalek::{Signature, VerifyingKey, Verifier};
use rust_decimal::prelude::*;

pub fn validate_signature_and_txid(transaction: &Transaction, check_txid: bool) -> bool {
    let preimage = Transaction::new(
        [0; 32], 
        *transaction.get_from(), 
        [0; 64],
        transaction.nonce, 
        transaction.created_at, 
        transaction.get_vin().to_vec(),
        transaction.get_vout().to_vec()
    );

    let binary_preimage = match binary::encode(&preimage) {
        Ok(data) => data,
        Err(_) => return false
    };
    let hash = sha256(&binary_preimage);

    if check_txid == true && hash != transaction.get_txid().clone() {
        return false;
    }

    let verifying_key = match VerifyingKey::from_bytes(transaction.get_from()) {
        Ok(data) => data,
        Err(_) => return false
    };
    let signature = Signature::from_bytes(transaction.get_signature());

    if let Ok(_) = verifying_key.verify(&hash, &signature) {
        true
    } else {
        false
    }
}

// pub fn validate_timestamp(transaction: &Transaction) -> bool {
//     current_epoch_number() == timestamp_to_epoch_number(transaction.get_created_at())
// }

pub async fn validate_utxo_exist_and_values(transaction: &Transaction, mempool: Arc<Mempool>) -> bool {
    let sum_vin_values = {
        let lock = mempool.get_utxos();
        let utxos = lock.read().await;
        
        let mut sum_vin_values: f64 = 0.0;

        for txinput in transaction.get_vin() {
            match utxos.get(&(txinput.get_txid().clone(), txinput.get_vout())) {
                Some(utxo) => {
                    sum_vin_values += utxo.get_value() as f64;
                    if utxo.get_public_key() != transaction.get_from() {
                        return false;
                    }
                },
                None => {
                    return false;
                }
            }
        }

        sum_vin_values
    };

    let sum_vout_values: f64 = transaction.get_vout()
    .iter()
    .map(|v| { v.value as f64 })
    .sum();

    let sum_vout_values = Decimal::from_f64(sum_vout_values)
        .unwrap()
        .round_dp(2);
    let sum_vin_values = Decimal::from_f64(sum_vin_values)
        .unwrap()
        .round_dp(2);

    sum_vout_values == sum_vin_values
}
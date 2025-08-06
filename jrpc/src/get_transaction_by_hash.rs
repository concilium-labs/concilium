use std::sync::Arc;
use concilium_core::{db::DB, jrpc::transaction::{GetTransactionByHash, GetTransactionByHashRequest, GetTransactionByHashResponse, GetTransactionByHashTXInput, GetTransactionByHashTXOutput}, transaction::Transaction};
use concilium_core_ext::{db::DBSupport, transaction::{get_transaction_by_hash::{response::GetTransactionByHashResponseSupport, transaction::GetTransactionByHashSupport, txinput::GetTransactionByHashTXInputSupport, txoutput::GetTransactionByHashTXOutputSupport}, transaction::TransactionSupport}};
use concilium_shared::binary;
use jsonrpsee::types::{ErrorObject, Params};

pub async fn handler(params: Params<'_>, db: Arc<DB>) -> Result<GetTransactionByHashResponse, ErrorObject<'static>> {
    let request: GetTransactionByHashRequest = match params.parse() {
        Ok(data) => data,
        Err(_) => {
            return Ok(failed_response());
        }
    };

    let txid = request.txid;
    if let Ok(data) = db.get(format!("transaction.{}", txid).as_str()) {
        if let Some(binary_transaction) = data {
            if let Ok(transaction) = binary::decode::<Transaction>(&binary_transaction) {
                let mut vin = Vec::new();
                let mut vout = Vec::new();

                for item in transaction.get_vin() {
                    vin.push(GetTransactionByHashTXInput::new(hex::encode(item.txid.clone()), item.vout));
                }
                
                for item in transaction.get_vout() {
                    vout.push(GetTransactionByHashTXOutput::new(item.value, hex::encode(item.public_key.clone())));
                }

                let transaction = GetTransactionByHash::new(
                    hex::encode(transaction.get_txid().clone()), 
                    hex::encode(transaction.get_from().clone()), 
                    hex::encode(transaction.get_signature().clone()), 
                    transaction.get_nonce(), 
                    transaction.get_created_at(), 
                    vin, 
                    vout
                );

                return Ok(GetTransactionByHashResponse::new(true, Some(transaction)));
            }
        }
    }

    return Ok(failed_response());
}

fn failed_response() -> GetTransactionByHashResponse {
    GetTransactionByHashResponse::new(false, None)
}
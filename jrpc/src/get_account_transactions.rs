use std::sync::Arc;
use concilium_core::{db::DB, jrpc::transaction::{GetAccountTransactionsRequest, GetAccountTransactionsResponse}, mempool::Mempool, transaction::Transaction};
use concilium_core_ext::{chain_state::ChainStateSupport, db::DBSupport, mempool::MempoolSupport, transaction::get_account_transactions::GetAccountTransactionsResponseSupport};
use concilium_shared::binary;
use jsonrpsee::types::{ErrorObject, Params};

pub async fn handler(params: Params<'_>, mempool: Arc<Mempool>, db: Arc<DB>) -> Result<GetAccountTransactionsResponse, ErrorObject<'static>> {
    let request: GetAccountTransactionsRequest = params.parse()?;

    let public_key: [u8; 32] = match hex::decode(request.public_key) {
        Ok(data) => match data.try_into() {
            Ok(converted) => converted,
            Err(_) => return Ok(failed_response())
        },
        Err(_) => return Ok(failed_response())
    };

    let chain_state_lock = mempool.get_chain_state();
    let chain_state = chain_state_lock.read().await;

    let transactions_ids = chain_state.get_transactions().get(&public_key);

    let mut transactions = Vec::new();
    if let Some(data) = transactions_ids {
        for item in data {
            if let Ok(binary) = db.get(format!("transaction.{}", hex::encode(*item)).as_str()) {
                if let Some(trx) = binary {
                    if let Ok(decoded) = binary::decode::<Transaction>(&trx) {
                        transactions.push(decoded);
                    }
                }
            }
        }
    }

    Ok(GetAccountTransactionsResponse::new(true, transactions))
}

fn failed_response() -> GetAccountTransactionsResponse {
    GetAccountTransactionsResponse::new(false, Vec::new())
}
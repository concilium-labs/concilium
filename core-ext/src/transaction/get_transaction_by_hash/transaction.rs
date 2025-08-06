use concilium_core::jrpc::transaction::{GetTransactionByHash, GetTransactionByHashTXInput, GetTransactionByHashTXOutput};

pub trait GetTransactionByHashSupport {
    fn new(txid: String, from: String, signature: String, nonce: u64, created_at: i64, vin: Vec<GetTransactionByHashTXInput>, vout: Vec<GetTransactionByHashTXOutput>) -> GetTransactionByHash;
}

impl GetTransactionByHashSupport for GetTransactionByHash {
    fn new(txid: String, from: String, signature: String, nonce: u64, created_at: i64, vin: Vec<GetTransactionByHashTXInput>, vout: Vec<GetTransactionByHashTXOutput>) -> GetTransactionByHash {
        Self {
            txid,
            from,
            signature,
            nonce,
            created_at,
            vin,
            vout
        }
    }
}

use concilium_core::jrpc::transaction::GetTransactionByHashTXInput;


pub trait GetTransactionByHashTXInputSupport {
    fn new(txid: String, vout: usize) -> GetTransactionByHashTXInput;
}

impl GetTransactionByHashTXInputSupport for GetTransactionByHashTXInput {
    fn new(txid: String, vout: usize) -> GetTransactionByHashTXInput {
        Self {
            txid,
            vout
        }
    }
}
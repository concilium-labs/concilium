use concilium_core::jrpc::transaction::SendRawTransactionRequestTXInput;

pub trait SendRawTransactionRequestTXInputSupport {
    fn new(txid: String, vout: usize) -> SendRawTransactionRequestTXInput;
    fn get_txid(&self) -> &String;
    fn get_vout(&self) -> usize;
}

impl SendRawTransactionRequestTXInputSupport for SendRawTransactionRequestTXInput {
    fn new(txid: String, vout: usize) -> SendRawTransactionRequestTXInput {
        Self {
            txid,
            vout
        }
    }

    fn get_txid(&self) -> &String {
        &self.txid
    }

    fn get_vout(&self) -> usize {
        self.vout
    }
}
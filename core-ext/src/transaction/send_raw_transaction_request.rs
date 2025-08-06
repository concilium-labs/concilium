use concilium_core::jrpc::transaction::{SendRawTransactionRequest, SendRawTransactionRequestTXInput, SendRawTransactionRequestTXOutput};

pub trait SendRawTransactionRequestSupport {
    fn new(from: String, signature: String, nonce: u64, created_at: i64, vin: Vec<SendRawTransactionRequestTXInput>, vout: Vec<SendRawTransactionRequestTXOutput>) -> SendRawTransactionRequest;
    fn get_from(&self) -> &String;
    fn get_signature(&self) -> &String;
    fn get_nonce(&self) -> u64;
    fn get_created_at(&self) -> i64;
    fn get_vin(&self) -> &Vec<SendRawTransactionRequestTXInput>;
    fn get_vout(&self) -> &Vec<SendRawTransactionRequestTXOutput>;
    fn set_vin(&mut self, vin: Vec<SendRawTransactionRequestTXInput>);
    fn set_vout(&mut self, vout: Vec<SendRawTransactionRequestTXOutput>);
}

impl SendRawTransactionRequestSupport for SendRawTransactionRequest {
    fn new(from: String, signature: String, nonce: u64, created_at: i64, vin: Vec<SendRawTransactionRequestTXInput>, vout: Vec<SendRawTransactionRequestTXOutput>) -> SendRawTransactionRequest {
        Self {
            from,
            signature,
            nonce,
            created_at,
            vin,
            vout
        }
    }

    fn get_from(&self) -> &String {
        &self.from
    }

    fn get_signature(&self) -> &String {
        &self.signature
    }

    fn get_nonce(&self) -> u64 {
        self.nonce
    }

    fn get_created_at(&self) -> i64 {
        self.created_at
    }

    fn get_vin(&self) -> &Vec<SendRawTransactionRequestTXInput> {
        &self.vin
    }

    fn get_vout(&self) -> &Vec<SendRawTransactionRequestTXOutput> {
        &self.vout
    }

    fn set_vin(&mut self, vin: Vec<SendRawTransactionRequestTXInput>) {
        self.vin = vin
    }

    fn set_vout(&mut self, vout: Vec<SendRawTransactionRequestTXOutput>) {
        self.vout = vout
    }
}

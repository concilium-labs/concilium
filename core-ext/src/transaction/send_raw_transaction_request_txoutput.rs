use concilium_core::jrpc::transaction::SendRawTransactionRequestTXOutput;

pub trait SendRawTransactionRequestTXOutputSupport {
    fn new(value: f32, public_key: String) -> SendRawTransactionRequestTXOutput;   
    fn get_value(&self) -> f32;
    fn get_public_key(&self) -> &String; 
}

impl SendRawTransactionRequestTXOutputSupport for SendRawTransactionRequestTXOutput {
    fn new(value: f32, public_key: String) -> SendRawTransactionRequestTXOutput {
        Self {
            value,
            public_key
        }
    }

    fn get_value(&self) -> f32 {
        self.value
    }

    fn get_public_key(&self) -> &String {
        &self.public_key
    }
}
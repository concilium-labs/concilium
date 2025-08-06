use concilium_core::jrpc::transaction::GetTransactionByHashTXOutput;

pub trait GetTransactionByHashTXOutputSupport {
    fn new(value: f32, public_key: String) -> GetTransactionByHashTXOutput;   
}

impl GetTransactionByHashTXOutputSupport for GetTransactionByHashTXOutput {
    fn new(value: f32, public_key: String) -> GetTransactionByHashTXOutput {
        Self {
            value,
            public_key
        }
    }
}

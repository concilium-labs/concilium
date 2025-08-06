use concilium_core::jrpc::transaction::{GetTransactionByHash, GetTransactionByHashResponse};

pub trait GetTransactionByHashResponseSupport {
    fn new(status: bool, transaction: Option<GetTransactionByHash>) -> GetTransactionByHashResponse;
}

impl GetTransactionByHashResponseSupport for GetTransactionByHashResponse {
    fn new(status: bool, transaction: Option<GetTransactionByHash>) -> GetTransactionByHashResponse {
        Self {
            status,
            transaction
        }
    }
}

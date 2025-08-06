use concilium_core::{jrpc::transaction::{GetAccountTransactionsRequest, GetAccountTransactionsResponse}, transaction::Transaction};

pub trait GetAccountTransactionsRequestSupport {
    fn new(public_key: String) -> GetAccountTransactionsRequest;
}

pub trait GetAccountTransactionsResponseSupport {
    fn new(status: bool, transactions: Vec<Transaction>) -> GetAccountTransactionsResponse;
}

impl GetAccountTransactionsRequestSupport for GetAccountTransactionsRequest {
    fn new(public_key: String) -> GetAccountTransactionsRequest {
        Self {
            public_key
        }
    }
}

impl GetAccountTransactionsResponseSupport for GetAccountTransactionsResponse {
    fn new(status: bool, transactions: Vec<Transaction>) -> GetAccountTransactionsResponse {
        Self {
            status,
            transactions
        }
    }    
}
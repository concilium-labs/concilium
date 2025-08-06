use serde::{Deserialize, Serialize};
use crate::transaction::Transaction;

#[derive(Serialize, Deserialize)]
pub struct SendRawTransactionRequestTXInput {
    pub txid: String,
    pub vout: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SendRawTransactionRequestTXOutput {
    pub value: f32,
    pub public_key: String
}

#[derive(Serialize, Deserialize)]
pub struct SendRawTransactionRequest {
    pub from: String, // public key
    pub signature: String,
    pub nonce: u64,
    pub created_at: i64,
    pub vin: Vec<SendRawTransactionRequestTXInput>,
    pub vout: Vec<SendRawTransactionRequestTXOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendRawTransactionResponse {
    pub status: bool,
    pub txid: String,
    pub accreditation_council_aggregated_signature: String,
    pub broadcast_aggregated_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransactionByHashTXInput {
    pub txid: String,
    pub vout: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransactionByHashTXOutput {
    pub value: f32,
    pub public_key: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransactionByHash {
    pub txid: String,
    pub from: String, // public key
    pub signature: String,
    pub nonce: u64,
    pub created_at: i64,
    pub vin: Vec<GetTransactionByHashTXInput>,
    pub vout: Vec<GetTransactionByHashTXOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransactionByHashResponse {
    pub status: bool,
    pub transaction: Option<GetTransactionByHash>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransactionByHashRequest {
    pub txid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAccountTransactionsRequest {
    pub public_key: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetAccountTransactionsResponse {
    pub status: bool,
    pub transactions: Vec<Transaction>
}
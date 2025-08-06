use serde_big_array::BigArray;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub txid: [u8; 32],
    pub from: [u8; 32], // public key
    #[serde(with = "BigArray")]
    pub signature: [u8; 64],
    pub nonce: u64,
    pub created_at: i64,
    pub vin: Vec<TXInput>,
    pub vout: Vec<TXOutput>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TXInput {
    pub txid: [u8; 32],
    pub vout: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TXOutput {
    pub value: f32,
    pub public_key: [u8; 32]
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SenderTransactionTemp {
    pub public_key: [u8; 32],
    pub epoch_number: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BroadcastTransactionTemp {
    pub transaction: Transaction,
    #[serde(with = "BigArray")]
    pub signature: [u8; 96],
}
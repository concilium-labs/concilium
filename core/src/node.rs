use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use crate::rpc::epoch::Client as EpochClient;
use crate::rpc::transaction::Client as TransactionClient;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SelfNode {
    pub id: u32,
    pub name: Vec<u8>,
    #[serde(with = "BigArray")]
    pub public_key: [u8; 48],
    pub private_key: [u8; 32],
    pub ip_address: [u8; 4],
    pub port: u16,
    pub version: Vec<u8>,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize)]
pub struct AwaitingConfirmationNode {
    pub id: u32,
    pub name: Vec<u8>,
    #[serde(with = "BigArray")]
    pub public_key: [u8; 48],
    pub ip_address: [u8; 4],
    pub port: u16,
    pub version: Vec<u8>,
    pub created_at: i64,
}

#[derive(Debug)]
pub struct ActiveNode {
    pub id: u32,
    pub name: Vec<u8>,
    pub public_key: [u8; 48],
    pub ip_address: [u8; 4],
    pub port: u16,
    pub version: Vec<u8>,
    pub created_at: i64,
    pub epoch_client: EpochClient,
    pub transaction_client: TransactionClient
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableNode {
    pub id: u32,
    pub name: Vec<u8>,
    #[serde(with = "BigArray")]
    pub public_key: [u8; 48],
    pub ip_address: [u8; 4],
    pub port: u16,
    pub version: Vec<u8>,
    pub created_at: i64,
}
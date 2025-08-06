use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GetAddressUtxosRequest {
    pub public_key: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetAddressUtxosVoutResponse {
    pub txid: [u8; 32],
    pub vout: usize,
    pub value: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetAddressUtxosResponse {
    pub status: bool,
    pub vout: Vec<GetAddressUtxosVoutResponse>
}

#[derive(Serialize)]
pub struct SendToAddressResponse {
    pub status: bool,
    pub message: String,
    pub txid: String,
    pub accreditation_council_aggregated_signature: String,
    pub broadcast_aggregated_signature: String,
}

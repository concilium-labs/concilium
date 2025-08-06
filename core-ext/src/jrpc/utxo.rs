use concilium_core::jrpc::utxo::{GetAddressUtxosResponse, GetAddressUtxosVoutResponse, SendToAddressResponse};

pub trait GetAddressUtxosResponseSupport {
    fn new(status: bool, vout: Vec<GetAddressUtxosVoutResponse>) -> GetAddressUtxosResponse;
}

pub trait GetAddressUtxosVoutResponseSupport {
    fn new(txid: [u8; 32], vout: usize, value: f32) -> GetAddressUtxosVoutResponse;
}

pub trait SendToAddressResponseSupport {
    fn new(status: bool, message: String, txid: String, accreditation_council_aggregated_signature: String, broadcast_aggregated_signature: String) -> SendToAddressResponse;
}

impl GetAddressUtxosResponseSupport for GetAddressUtxosResponse {
    fn new(status: bool, vout: Vec<GetAddressUtxosVoutResponse>) -> GetAddressUtxosResponse {
        Self {
            status,
            vout
        }
    }
}

impl GetAddressUtxosVoutResponseSupport for GetAddressUtxosVoutResponse {
    fn new(txid: [u8; 32], vout: usize, value: f32) -> GetAddressUtxosVoutResponse {
        Self {
            txid,
            vout,
            value
        }
    }
}

impl SendToAddressResponseSupport for SendToAddressResponse {
    fn new(status: bool, message: String, txid: String, accreditation_council_aggregated_signature: String, broadcast_aggregated_signature: String) -> SendToAddressResponse {
        Self {
            status,
            message,
            txid,
            accreditation_council_aggregated_signature,
            broadcast_aggregated_signature
        }
    }
}

use concilium_core::jrpc::transaction::SendRawTransactionResponse;

pub trait SendRawTransactionResponseSupport {
    fn new(status: bool, txid: String, accreditation_council_aggregated_signature: String, broadcast_aggregated_signature: String) -> SendRawTransactionResponse;
}

impl SendRawTransactionResponseSupport for SendRawTransactionResponse {
    fn new(status: bool, txid: String, accreditation_council_aggregated_signature: String, broadcast_aggregated_signature: String) -> SendRawTransactionResponse {
        Self {
            status,
            txid,
            accreditation_council_aggregated_signature,
            broadcast_aggregated_signature,
        }
    }
}

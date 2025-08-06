use std::sync::Arc;
use concilium_core::{jrpc::utxo::{GetAddressUtxosRequest, GetAddressUtxosResponse, GetAddressUtxosVoutResponse}, mempool::Mempool};
use concilium_core_ext::{jrpc::utxo::{GetAddressUtxosResponseSupport, GetAddressUtxosVoutResponseSupport}, mempool::MempoolSupport};
use jsonrpsee::types::{ErrorObject, Params};

pub async fn handler(params: Params<'_>, mempool: Arc<Mempool>) -> Result<GetAddressUtxosResponse, ErrorObject<'static>> {
    let request: GetAddressUtxosRequest = params.parse()?;

    let lock = mempool.get_utxos();
    let utxos = lock.read().await;

    let public_key: [u8; 32] = match hex::decode(request.public_key) {
        Ok(data) => match data.try_into() {
            Ok(converted) => converted,
            Err(_) => return Ok(failed_response())
        },
        Err(_) => return Ok(failed_response())
    };

    let mut vout = Vec::new();

    for item in utxos.iter() {
        if item.1.public_key == public_key {
            vout.push(GetAddressUtxosVoutResponse::new(item.0.0, item.0.1, item.1.value));
        }
    }

    Ok(GetAddressUtxosResponse::new(true, vout))
}

fn failed_response() -> GetAddressUtxosResponse {
    GetAddressUtxosResponse::new(false, Vec::new())
}
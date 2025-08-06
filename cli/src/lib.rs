use std::env;
use blst::min_pk::SecretKey;
use chrono::Utc;
use concilium_core::jrpc::utxo::{GetAddressUtxosResponse, SendToAddressResponse};
use concilium_core::transaction::{TXInput, TXOutput, Transaction};
use concilium_core_ext::jrpc::utxo::SendToAddressResponseSupport;
use concilium_core_ext::transaction::send_raw_transaction_request::SendRawTransactionRequestSupport;
use concilium_core_ext::transaction::send_raw_transaction_request_txinput::SendRawTransactionRequestTXInputSupport;
use concilium_core_ext::transaction::send_raw_transaction_request_txoutput::SendRawTransactionRequestTXOutputSupport;
use concilium_core_ext::transaction::transaction::TransactionSupport;
use concilium_core_ext::transaction::txinput::TXInputSupport;
use concilium_core_ext::transaction::txoutput::TXOutputSupport;
use concilium_error::Error;
use concilium_shared::binary;
use concilium_shared::sha::sha256;
use ed25519_dalek::ed25519::signature::SignerMut;
use ed25519_dalek::{Signature, SigningKey, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH};
use rand::{rngs::OsRng, RngCore};
use concilium_core::jrpc::transaction::{GetTransactionByHashResponse, SendRawTransactionRequest, SendRawTransactionRequestTXInput, SendRawTransactionRequestTXOutput, SendRawTransactionResponse};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::params::ObjectParams;
use jsonrpsee::http_client::HttpClient;
use std::time::Duration;
use serde_json::json;

pub fn get_new_node_wallet_handler() -> Result<(String, String), Error> {
    let mut rng = OsRng;
    let mut ikm : [u8; 32] = [0; 32];
    rng.fill_bytes(&mut ikm);

    let private_key = SecretKey::key_gen(&ikm, &[])?;
    let public_key = private_key.sk_to_pk();

    Ok((hex::encode(public_key.to_bytes()), hex::encode(private_key.to_bytes())))
}

pub fn get_new_user_wallet_handler() -> (String, String) {
    let mut rng = OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut rng);

    let public_key: [u8; PUBLIC_KEY_LENGTH] = signing_key.verifying_key().to_bytes();
    let private_key: [u8; SECRET_KEY_LENGTH] = signing_key.to_bytes();

    (hex::encode(public_key), hex::encode(private_key))
}

pub async fn get_transaction_info_handler(txid: String) -> Result<GetTransactionByHashResponse, Error> {
    let dst = format!("http://{}:{}", env::var("NODE_IP_ADDRESS")?, env::var("NODE_JSON_RPC_PORT")?);
    let client = HttpClient::builder().request_timeout(Duration::from_secs(10)).build(dst)?;
    let raw_params = json!({
        "txid": txid, 
    });
    
    let mut obj_params = ObjectParams::new();
    if let serde_json::Value::Object(map) = raw_params {
        for (key, value) in map {
            obj_params.insert(key.as_str(), value)?;
        }
    }

    Ok(client.request::<GetTransactionByHashResponse, _>("get_transaction_by_hash", obj_params).await?)
}

pub async fn send_to_address_handler(sender_private_key: String, receiver_public_key: String, amount: f32) -> Result<SendToAddressResponse, Error> {
    let private_key: [u8; 32] = hex::decode(sender_private_key)?.try_into()?;
    let mut signing_key: SigningKey = SigningKey::from_bytes(&private_key);
    let sender_public_key = signing_key.verifying_key();
    let sender_public_key = sender_public_key.to_bytes();

    let dst = format!("http://{}:{}", env::var("NODE_IP_ADDRESS")?, env::var("NODE_JSON_RPC_PORT")?);
    let client = HttpClient::builder().request_timeout(Duration::from_secs(10)).build(dst)?;
    let raw_params = json!({
        "public_key": hex::encode(sender_public_key), 
    });
    
    let mut obj_params = ObjectParams::new();
    if let serde_json::Value::Object(map) = raw_params {
        for (key, value) in map {
            obj_params.insert(key.as_str(), value)?;
        }
    }

    let utxos = client.request::<GetAddressUtxosResponse, _>("get_address_utxos", obj_params).await?;

    if utxos.status == false {
        return Err(Error::new("Internal Error"))
    }

    let mut needed_utxos = Vec::new();
    let mut needed_amount: f32 = 0.0;

    for item in utxos.vout {
        if needed_amount >= amount {
            break;
        } else {
            needed_amount += item.value;
            needed_utxos.push(item);
        }
    }

    if needed_amount < amount {
        return Err(Error::new("Your balance is not sufficient"))
    }

    let mut vin = Vec::new();

    for item in needed_utxos {
        vin.push(TXInput::new(item.txid, item.vout));
    } 
    
    let mut vout = Vec::new();

    let receiver_public_key: [u8; 32] = hex::decode(receiver_public_key)?.try_into()?;
    if needed_amount == amount {
        vout.push(TXOutput::new(amount, receiver_public_key));
    } else {
        vout.push(TXOutput::new(amount, receiver_public_key));
        vout.push(TXOutput::new(needed_amount - amount, sender_public_key));

    }

    let created_at = Utc::now().timestamp();
    let nonce = rand::random::<u64>();

    let preimage = Transaction::new(
        [0; 32], 
        sender_public_key,
        [0; 64],
        nonce, 
        created_at, 
        vin.clone(),
        vout.clone()
    );

    let binary_preimage = binary::encode(&preimage).unwrap();
    let txid = sha256(&binary_preimage);
    let signature: Signature = signing_key.sign(&txid);
    let mut transaction_vin = Vec::new();
    let mut transaction_vout = Vec::new();

    for item in vin {
        transaction_vin.push(SendRawTransactionRequestTXInput::new(
            hex::encode(item.txid), 
            item.vout
        ));
    }
    
    for item in vout {
        transaction_vout.push(SendRawTransactionRequestTXOutput::new(
            item.value,
            hex::encode(item.public_key)
        ));
    }

    let transaction = SendRawTransactionRequest::new(
        hex::encode(sender_public_key), 
        hex::encode(signature.to_bytes()), 
        nonce, 
        created_at, 
        transaction_vin, 
        transaction_vout
    );

    let raw_params = json!({
        "from": transaction.from,
        "signature": transaction.signature,
        "nonce": transaction.nonce,
        "created_at": transaction.created_at,
        "vin": transaction.vin,
        "vout": transaction.vout,
    });

    let mut obj_params = ObjectParams::new();
    if let serde_json::Value::Object(map) = raw_params {
        for (key, value) in map {
            obj_params.insert(key.as_str(), value).unwrap();
        }
    }

    let response: SendRawTransactionResponse = client.request("send_raw_transaction", obj_params).await?;

    if response.status == true {
        Ok(SendToAddressResponse::new(
            true, 
            String::from("Successful transaction"), 
            response.txid, 
            response.accreditation_council_aggregated_signature,
            response.broadcast_aggregated_signature 
        ))
    } else {
        Err(Error::new("Unsuccessful transaction"))
    }
}
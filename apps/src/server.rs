use ahash::AHashMap;
use concilium_core_ext::{db::DBSupport, epoch::{EpochPoolSupport, EpochSupport}, jrpc::rpc_module_context::RpcModuleContextSupport, mempool::{active_nodes::ActiveNodesSupport, MempoolSupport}, node::{active_node::ActiveNodeSupport, self_node::SelfNodeSupport, serializable_node::SerializableNodeSupport}, temporary_node_ids::TemporaryNodeIdsSupport};
use concilium_jrpc::{
    get_address_utxos::handler as get_address_utxos_handler, get_transaction_by_hash::handler as get_transaction_by_hash_handler, send_raw_transaction::handler as send_raw_transaction_handler, get_account_transactions::handler as get_account_transactions_handler
};
use concilium_proto_defs::{
    connection::connection_server::ConnectionServer as ConnectionService,
    epoch::epoch_server::EpochServer as EpochService, 
    identifier::identifier_server::IdentifierServer as IdentifierService,
    transaction::transaction_server::TransactionServer as TransactionService
};
use concilium_core::{
    db::DB, epoch::Epoch, jrpc::rpc_module_context::RpcModuleContext, mempool::Mempool, node::{
        ActiveNode,
        SerializableNode
    }, 
    rpc::{
        connection::Client as ConnectionClient, 
        epoch::Client as EpochClient, 
        identifier::Client as IdentifierClient,
        transaction::Client as TransactionClient,
    }
};
use concilium_rpc::{
    connection::{
        client::ClientSupport as ConnectionClientSupport,
        server::{
            ServerSupport as ConnectionServerSupport,
            Server as ConnectionServer
        },
    },
    identifier::{
        client::ClientSupport as IdentifierClientSupport,
        server::{
            ServerSupport as IdentifierServerSupport,
            Server as IdentifierServer
        }
    },
    epoch::{
        client::ClientSupport as EpochClientSupport,
        server::{
            ServerSupport as EpochServerSupport,
            Server as EpochServer
        }
    },
    transaction::{
        client::ClientSupport as TransactionClientSupport,
        server::{
            ServerSupport as TransactionServerSupport,
            Server as TransactionServer
        }
    },
};
use concilium_shared::{
    binary, chacha20::generate_random_number_by_seed, epoch::current_epoch_number, ip::ipv4_to_string, sha::sha256, BOOTSTRAP_NODES, DST
};
use concilium_log as log;
use concilium_genesis::{load_genesis_transactions, load_transactions};
use jsonrpsee::server::{RpcModule, ServerBuilder as JsonrpseeServer, ServerConfigBuilder};
use hyper::Method;
use tower_http::cors::{Any, CorsLayer};
use rand::{rng, Rng};
use std::{env, net::SocketAddr, sync::Arc, u16};
use blst::min_pk::{AggregatePublicKey, AggregateSignature, PublicKey, SecretKey, Signature};
use blst::BLST_ERROR;
use concilium_error::Error;
use tonic::transport::Server as TonicServer;
use chrono::Utc;
use tokio::{
    sync::Mutex, task::JoinHandle, time::{
        sleep, sleep_until, Duration, Instant
    }
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("ENV File Not Found");
    let db = Arc::new(DB::new().expect("Database Error"));
    let mempool = Arc::new(Mempool::new().expect("Mempool Error"));

    if db.exist("included_genesis_transactions") {
        load_transactions(Arc::clone(&mempool), Arc::clone(&db)).await;
    } else {
        load_genesis_transactions(Arc::clone(&mempool), Arc::clone(&db)).await;
    }

    connect_to_network(Arc::clone(&mempool)).await.unwrap();

    handling_epoch(Arc::clone(&mempool));    

    let _ = tokio::join!(
        rpc_server_handler(Arc::clone(&mempool), Arc::clone(&db)),
        json_rpc_server_handler(Arc::clone(&mempool), Arc::clone(&db))
    );
}

async fn connect_to_network(mempool: Arc<Mempool>) -> Result<(), Error> {
    loop {
        let result = do_connect_to_network(Arc::clone(&mempool)).await;

        match result {
            Ok(_) => {
                break Ok(());
            }
            Err(e) => {
                log::error(e.get_message()).await.ok();
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        }
    }
}

async fn do_connect_to_network(mempool: Arc<Mempool>) -> Result<(), Error> {
    let mut responses = Vec::new();
    let mut is_bootstrap_node = false;
    let mut bootstrap_node_public_keys = Vec::new();

    let slef_node_lock = mempool.get_self_node();
    let mut self_node = slef_node_lock.write().await;

    let bootstrap_node_signature_lock = mempool.get_bootstrap_node_signature();
    let mut bootstrap_node_signature = bootstrap_node_signature_lock.lock().await;

    for node in BOOTSTRAP_NODES {
        if node[0] == hex::encode(self_node.get_public_key()).as_str() {
            is_bootstrap_node = true;
            continue;
        }
        
        let mut client = match IdentifierClient::connect(node[1]).await {
            Ok(client) => client,
            Err(_) => continue
        };
        if let Ok(data) = client.get_id(self_node.get_self()).await {
            responses.push(data.into_inner());
            bootstrap_node_public_keys.push(node[0]);
        }
    }

    if responses.len() == 0 {
        if is_bootstrap_node {
            if let Ok(private_key) = SecretKey::from_bytes(self_node.get_private_key()) {
                let message = binary::encode(
                    &SerializableNode::new
                    (
                        1, 
                        self_node.get_name().to_owned(), 
                        *self_node.get_public_key(), 
                        *self_node.get_ip_address(), 
                        self_node.get_port(), 
                        self_node.get_version().to_owned(), 
                        self_node.get_created_at()
                    )
                )?;

                create_initial_entropies(Arc::clone(&mempool), 1).await;

                let agg_sig = AggregateSignature::aggregate(&vec![&private_key.sign(&message, DST, &[])], false)?.to_signature();

                *bootstrap_node_signature = agg_sig.to_bytes();

                self_node.set_id(1);

                return Ok(());
            }  
            Err(Error::new("Is Bootstrap Node Error"))
        } else {
            Err(Error::new("Is Bootstrap Node Error"))
        }
    } else {
        let id =  responses[0].id;

        let message =  binary::encode(
            &SerializableNode::new
            (
                id, 
                self_node.get_name().to_owned(), 
                *self_node.get_public_key(), 
                *self_node.get_ip_address(), 
                self_node.get_port(), 
                self_node.get_version().to_owned(), 
                self_node.get_created_at()
            )
        )?;

        let signatures: Vec<Signature> = responses.iter()
        .filter_map(|item| {
            Signature::from_bytes(&item.signature).ok()
        })
        .collect();
        let signatures = signatures.iter().collect::<Vec<&Signature>>();

        let public_keys: Vec<PublicKey> = bootstrap_node_public_keys.iter()
        .filter_map(|item| {
            let decoded = hex::decode(item).ok()?;
            PublicKey::from_bytes(&decoded).ok()
        })
        .collect();
        let public_keys = public_keys.iter().collect::<Vec<&PublicKey>>();

        let agg_sig = AggregateSignature::aggregate(&signatures, false)?.to_signature();
        let agg_pub = AggregatePublicKey::aggregate(&public_keys, false)?.to_public_key();

        let agg_sig_bytes = agg_sig.to_bytes();
        if agg_sig.verify(false, &message, DST, &[], &agg_pub, true) == BLST_ERROR::BLST_SUCCESS {            
            let mut success_statuses = Vec::new();
            let mut nodes: Vec<SerializableNode> = Vec::new();

            for node in BOOTSTRAP_NODES {
                let mut client = match IdentifierClient::connect(node[1]).await {
                    Ok(client) => client,
                    Err(_) => continue
                };
                
                if let Ok(data) = client.validate_id(&message, &agg_sig_bytes).await {
                    let response = data.into_inner();

                    if let Ok(n) = binary::decode::<Vec<SerializableNode>>(&response.nodes) {
                        if n.len() > nodes.len() {
                            nodes = n;
                        }
                    }
                    
                    if response.status == true {
                        success_statuses.push(response.status);
                    }
                }
            }          

            if success_statuses.len() == public_keys.len() {
                create_initial_entropies(Arc::clone(&mempool), id).await;

                self_node.set_id(id);
                *bootstrap_node_signature = agg_sig_bytes;

                let active_nodes_lock = mempool.get_active_nodes();
                let mut active_nodes = active_nodes_lock.write().await;
                
                for node in nodes {
                    let node_address = &format!("{}:{}", ipv4_to_string(node.get_ip_address()), node.get_port());
                    if let Ok(mut client) = ConnectionClient::connect(node_address).await {
                        if let Ok(is_connected) = client.initial_connect(self_node.get_self(), &agg_sig_bytes).await{
                            if is_connected.get_ref().status == true {
                                let epoch_client = EpochClient::connect(node_address).await.unwrap();
                                let transaction_client = TransactionClient::connect(node_address).await.unwrap();

                                active_nodes.insert_or_update(Arc::new(
                                    ActiveNode::new(
                                        node.id, 
                                        node.name, 
                                        node.public_key, 
                                        node.ip_address, 
                                        node.port, 
                                        node.version, 
                                        node.created_at, 
                                        epoch_client,
                                        transaction_client
                                    )
                                ));
                            }
                        } 
                    }          
                }
                
                Ok(())
            } else {
                Err(Error::new("Public Keys Len Error"))
            }
        } else {
            Err(Error::new("Signature Error"))
        }
    }
}

async fn create_initial_entropies(mempool: Arc<Mempool>, last_node_id: u32) {
    let current_epoch_number = u64::try_from(current_epoch_number()).unwrap();
    
    let lock = mempool.get_epoch_pool().get_write();
    let mut epoch_pool = lock.lock().await;

    epoch_pool.insert(current_epoch_number, Arc::new(Epoch::new(current_epoch_number, last_node_id, [0; 32], Vec::new(), AHashMap::new())));
    epoch_pool.insert(current_epoch_number + 1, Arc::new(Epoch::new(current_epoch_number + 1, last_node_id, [0; 32], Vec::new(), AHashMap::new())));
    epoch_pool.insert(current_epoch_number + 2, Arc::new(Epoch::new(current_epoch_number + 2, last_node_id, [0; 32], Vec::new(), AHashMap::new())));
    epoch_pool.publish();
}

fn handling_epoch(mempool: Arc<Mempool>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let nodes: Arc<Mutex<AHashMap<u32, Arc<ActiveNode>>>> = Arc::new(Mutex::new(AHashMap::new()));

        let nodes_clone = Arc::clone(&nodes);
        let mempool_clone = Arc::clone(&mempool);
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(5)).await;
                let active_nodes_lock = mempool_clone.get_active_nodes();
                let active_nodes = active_nodes_lock.read().await;

                let mut nodes = nodes_clone.lock().await;
                *nodes = active_nodes.get_nodes_by_id().clone();
            }
        });
        
        let now = Utc::now();
        let seconds = u64::try_from(now.timestamp()).unwrap();
        let nanos = now.timestamp_subsec_nanos() as u64;

        let elapsed_in_cycle = (seconds % 12) * 1_000_000_000 + nanos;
        let nanos_to_next_cycle = if elapsed_in_cycle == 0 {
            0
        } else {
            (12 * 1_000_000_000) - elapsed_in_cycle
        };

        sleep_until(Instant::now() + Duration::from_nanos(nanos_to_next_cycle)).await;

        loop {
            let cycle_start = Instant::now();

            for stage in 0..3 {
                let stage_start = cycle_start + Duration::from_secs(4 * stage);
                let stage_end = stage_start + Duration::from_secs(4);
                let current_epoch_number = u64::try_from(current_epoch_number()).unwrap();

                sleep_until(stage_start).await;

                let mut stage_0_started = false;
                let mut stage_1_started = false;
                let mut stage_2_started = false;
                loop {
                    let now = Instant::now();
                    if now >= stage_end {
                        break;
                    }

                    if stage == 0 && stage_0_started == false {
                        stage_0_started = true;
                        let nodes = Arc::clone(&nodes);
                        let epoch_pool_read = mempool.get_epoch_pool().get_read();
                        let epoch_pool_write = mempool.get_epoch_pool().get_write();
                        let mut epoch_pool_write_guard = epoch_pool_write.lock().await;
                        
                        if let None = epoch_pool_read.get(&(current_epoch_number + 2)) {
                            let last_node_id = match epoch_pool_read.get(&(current_epoch_number + 1)) {
                                Some(data) => data.get_last_node_id(),
                                None => {
                                    let last_node_id = epoch_pool_read.get(&current_epoch_number).unwrap().get_last_node_id();

                                    epoch_pool_write_guard.insert(current_epoch_number + 1, Arc::new(Epoch::new(current_epoch_number + 1, last_node_id, [0; 32], Vec::new(), AHashMap::new())));
                                    epoch_pool_write_guard.publish();

                                    last_node_id
                                }
                            };

                            epoch_pool_write_guard.insert(current_epoch_number + 2, Arc::new(Epoch::new(current_epoch_number + 2, last_node_id, [0; 32], Vec::new(), AHashMap::new())));
                            epoch_pool_write_guard.publish();
                        }
                        
                        if let Some(epoch) = epoch_pool_read.get(&(current_epoch_number + 1)) {
                            let random_number = rng().random::<u64>();
                            let mut random_numbers = epoch.get_random_numbers().to_vec();
                            random_numbers.push(random_number);

                            epoch_pool_write_guard.update(epoch.get_id().clone(), Arc::new(Epoch::new(epoch.get_id().clone(), epoch.get_last_node_id().clone(), epoch.get_final_hash().clone(), random_numbers, epoch.get_hashes().clone())));
                            epoch_pool_write_guard.publish();

                            tokio::spawn(async move {
                                let nodes = nodes.lock().await;

                                for (_, node) in nodes.iter() {
                                    node.epoch_client.initial_request(current_epoch_number + 1, random_number).await.ok();
                                }
                            });
                        };
                    }
                    
                    if stage == 1 && stage_1_started == false {
                        stage_1_started = true;
                        let nodes = Arc::clone(&nodes);
                        let epoch_pool_read = mempool.get_epoch_pool().get_read();
                        let epoch_pool_write = mempool.get_epoch_pool().get_write();
                        let mut epoch_pool_write_guard = epoch_pool_write.lock().await;
                        
                        if let Some(epoch) = epoch_pool_read.get(&(current_epoch_number + 1)) {
                            let mut random_numbers = epoch.get_random_numbers().to_vec();
                            let mut hashes = epoch.get_hashes().clone();
                            
                            random_numbers.sort();
                            
                            let binary = binary::encode(&epoch.get_random_numbers().to_vec()).unwrap();

                            let hash = sha256(&binary);

                            if let Some(h) = hashes.get_mut(&hash) {
                                *h += 1;
                            } else {
                                hashes.insert(hash, 1);
                            }

                            epoch_pool_write_guard.update(epoch.get_id().clone(), Arc::new(Epoch::new(epoch.get_id().clone(), epoch.get_last_node_id().clone(), epoch.get_final_hash().clone(), random_numbers, hashes)));
                            epoch_pool_write_guard.publish();

                            tokio::spawn(async move {
                                let nodes = nodes.lock().await;

                                for (_, node) in nodes.iter() {
                                    node.epoch_client.sync_request(current_epoch_number + 1, &hash).await.ok();
                                }
                            });
                        };
                    }
                    
                    if stage == 2 && stage_2_started == false {

                        stage_2_started = true;
                        let epoch_pool_read = mempool.get_epoch_pool().get_read();
                        let epoch_pool_write = mempool.get_epoch_pool().get_write();
                        let mut epoch_pool_write_guard = epoch_pool_write.lock().await;

                        if let Some(epoch) = epoch_pool_read.get(&(current_epoch_number + 1)) {
                            let max_key = epoch.get_hashes().iter()
                            .max_by_key(|&(_, value)| value)
                            .map(|(key, _)| key);

                            let final_hash = match max_key {
                                Some(m) => m.clone(),
                                None => [0; 32]
                            };

                            epoch_pool_write_guard.update(epoch.get_id().clone(), Arc::new(Epoch::new(epoch.get_id().clone(), epoch.get_last_node_id().clone(), final_hash, epoch.get_random_numbers().to_vec(), epoch.get_hashes().clone())));
                            epoch_pool_write_guard.publish();

                            let node_ids = generate_random_number_by_seed(final_hash, epoch.get_last_node_id(), epoch.get_last_node_id());

                            let mut ids = AHashMap::new();
                            for (i, &item) in node_ids.iter().enumerate() {
                                ids.insert(item, (i + 1) as u32);
                            }

                            let temporary_node_ids_read= mempool.get_temporary_node_ids().get_read();
                            let temporary_node_ids_write = mempool.get_temporary_node_ids().get_write();
                            let mut temporary_node_ids_write_guard = temporary_node_ids_write.lock().await;

                            temporary_node_ids_write_guard.insert(current_epoch_number + 1, Arc::new(ids));
                            temporary_node_ids_write_guard.publish();   

                            let temporary_before_cycle_ids: Vec<u64> = temporary_node_ids_read.get_keys().unwrap()
                            .iter()
                            .filter(|&&number| {
                                number <= current_epoch_number - 49
                            })
                            .map(|&number| {
                                number
                            })
                            .collect();
                            let epoch_before_cycle_ids: Vec<u64> = epoch_pool_read.get_keys().unwrap()
                                .iter()
                                .filter(|&&number| {
                                    number <= current_epoch_number - 48
                                })
                                .map(|&number| {
                                    number
                                })
                                .collect();

                            for item in temporary_before_cycle_ids {
                                temporary_node_ids_write_guard.remove(item);
                            }
                            for item in epoch_before_cycle_ids {
                                epoch_pool_write_guard.remove(item);
                            }

                            temporary_node_ids_write_guard.publish();
                            epoch_pool_write_guard.publish();
                        };
                    }


                    sleep(Duration::from_millis(50)).await;
                }
            }

            sleep_until(cycle_start + Duration::from_secs(12)).await;
        }
    })
}

fn rpc_server_handler(mempool: Arc<Mempool>, db: Arc<DB>) -> JoinHandle<()> {
    tokio::spawn(async move {
        println!("RPC server is running on[http://127.0.0.1:{}]", env::var("NODE_RPC_PORT").unwrap());
        

        TonicServer::builder()
        .concurrency_limit_per_connection(u16::MAX as usize)
        .add_service(IdentifierService::new(IdentifierServer::new(Arc::clone(&mempool))))
        .add_service(EpochService::new(EpochServer::new(Arc::clone(&mempool))))
        .add_service(ConnectionService::new(ConnectionServer::new(Arc::clone(&mempool))))
        .add_service(TransactionService::new(TransactionServer::new(Arc::clone(&mempool), Arc::clone(&db))))
        .serve(format!("127.0.0.1:{}", env::var("NODE_RPC_PORT").unwrap()).parse().unwrap())
        .await
        .unwrap();  
    })
}

fn json_rpc_server_handler(mempool: Arc<Mempool>, db: Arc<DB>) -> JoinHandle<()> {
    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        println!("JRPC server is running on[http://127.0.0.1:{}]", env::var("NODE_JSON_RPC_PORT").unwrap());
        println!("i'm ready...");
        
        let config = ServerConfigBuilder::default()
        .max_connections(u16::MAX as u32)
        .build();

        let cors = CorsLayer::new()
        .allow_methods([Method::POST])
        .allow_origin(Any)
        .allow_headers([hyper::header::CONTENT_TYPE]);
        let middleware = tower::ServiceBuilder::new().layer(cors);
    
        let mut module = RpcModule::new(RpcModuleContext::new(Arc::clone(&mempool), Arc::clone(&db)));
        module.register_async_method("send_raw_transaction", |params, ctx, _| async move {
            send_raw_transaction_handler(params, ctx.get_mempool(), ctx.get_db()).await
        }).unwrap();
        
        module.register_async_method("get_transaction_by_hash", |params, ctx, _| async move {
            get_transaction_by_hash_handler(params, ctx.get_db()).await
        }).unwrap();
        
        module.register_async_method("get_account_transactions", |params, ctx, _| async move {
            get_account_transactions_handler(params, ctx.get_mempool(), ctx.get_db()).await
        }).unwrap();
        
        module.register_async_method("get_address_utxos", |params, ctx, _| async move {
            get_address_utxos_handler(params, ctx.get_mempool()).await
        }).unwrap();

        let server = JsonrpseeServer::default()
        .set_config(config)
        .set_http_middleware(middleware)
        .build(format!("127.0.0.1:{}", env::var("NODE_JSON_RPC_PORT").unwrap()).parse::<SocketAddr>().unwrap())
        .await.unwrap();
        
        server.start(module).stopped().await;
    })
}
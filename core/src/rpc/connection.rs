use std::sync::Arc;
use tonic::transport::Channel;
use concilium_proto_defs::connection::connection_client::ConnectionClient;
use crate::mempool::Mempool;

pub struct Client {
    pub client: ConnectionClient<Channel>
}

pub struct Server {
    pub mempool: Arc<Mempool>,
}

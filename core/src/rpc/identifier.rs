use std::sync::Arc;
use tonic::transport::Channel;
use concilium_proto_defs::identifier::identifier_client::IdentifierClient;
use crate::mempool::Mempool;

pub struct Client {
    pub client: IdentifierClient<Channel>
}

pub struct Server {
    pub mempool: Arc<Mempool>,
}


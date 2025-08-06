use concilium_core_ext::node::self_node::SelfNodeSupport;
use concilium_error::Error;
use concilium_core::{
    rpc::connection::Client,
    node::SelfNode
};
use concilium_proto_defs::connection::{connection_client::ConnectionClient, InitialConnectRequest, InitialConnectResponse};

#[tonic::async_trait]
pub trait ClientSupport {
    async fn connect(dst: &str) -> Result<Client, Error>;
    async fn initial_connect(&mut self, self_node: &SelfNode, signature: &[u8; 96]) -> Result<tonic::Response<InitialConnectResponse>, Error>;
}

#[tonic::async_trait]
impl ClientSupport for Client {
    async fn connect(dst: &str) -> Result<Client, Error> {
        let client = ConnectionClient::connect(format!("http://{}", dst)).await?;

        Ok(
            Self {
                client,
            }
        )
    }
    
    async fn initial_connect(&mut self, self_node: &SelfNode, signature: &[u8; 96]) -> Result<tonic::Response<InitialConnectResponse>, Error> {
        Ok(
            self.client.initial_connect(InitialConnectRequest {
                id: self_node.get_id(),
                name: self_node.get_name().to_vec(),
                public_key: self_node.get_public_key().to_vec(),
                ip_address: self_node.get_ip_address().to_vec(),
                port: self_node.get_port() as u32,
                version: self_node.get_version().to_vec(),
                created_at: self_node.get_created_at(),
                signature: signature.to_vec()
            }).await?
        )
    }
}
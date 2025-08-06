use concilium_core::{node::SelfNode, rpc::identifier::Client};
use concilium_core_ext::node::self_node::SelfNodeSupport;
use concilium_error::Error;
use tonic::Response;
use concilium_proto_defs::identifier::{
    GetIdRequest, 
    GetIdResponse, 
    ValidateIdRequest, 
    ValidateIdResponse,
    identifier_client::IdentifierClient
};

#[tonic::async_trait]
pub trait ClientSupport {
    async fn connect(dst: &str) -> Result<Client, Error>;
    async fn get_id(&mut self, self_node: &SelfNode) -> Result<tonic::Response<GetIdResponse>, Error>;
    async fn validate_id(&mut self, message: &[u8], signatures: &[u8; 96]) -> Result<tonic::Response<ValidateIdResponse>, Error>;
}

#[tonic::async_trait]
impl ClientSupport for Client {
    async fn connect(dst: &str) -> Result<Client, Error> {
        let client = IdentifierClient::connect(format!("http://{}", dst)).await?;

        Ok(
            Self {
                client
            }
        )
    }
    
    async fn get_id(&mut self, self_node: &SelfNode) -> Result<Response<GetIdResponse>, Error> {
        Ok(
            self.client.get_id(
                GetIdRequest
                {
                    name: self_node.get_name().to_vec(),
                    public_key: self_node.get_public_key().to_vec(),
                    ip_address: self_node.get_ip_address().to_vec(),
                    port: self_node.get_port() as u32,
                    version: self_node.get_version().to_vec(),
                    created_at: self_node.get_created_at()
                }
            ).await?
        )
    }
    
    async fn validate_id(&mut self, message: &[u8], signatures: &[u8; 96]) -> Result<Response<ValidateIdResponse>, Error> {
        Ok(
            self.client.validate_id(ValidateIdRequest {
                message: message.to_vec(),
                signatures: signatures.to_vec()
            }).await?
        )
    }
}
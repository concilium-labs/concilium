use std::sync::Arc;
use crate::{db::DB, mempool::Mempool};

pub struct RpcModuleContext {
    pub mempool: Arc<Mempool>,
    pub db: Arc<DB>,
}
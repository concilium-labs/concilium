use std::sync::Arc;
use concilium_core::{db::DB, jrpc::rpc_module_context::RpcModuleContext, mempool::Mempool};

pub trait RpcModuleContextSupport {
    fn new(mempool: Arc<Mempool>, db: Arc<DB>) -> RpcModuleContext;
    fn get_mempool(&self) -> Arc<Mempool>;
    fn get_db(&self) -> Arc<DB>;
}

impl RpcModuleContextSupport for RpcModuleContext {
    fn new(mempool: Arc<Mempool>, db: Arc<DB>) -> RpcModuleContext {
        Self {
            mempool,
            db
        }
    }

    fn get_mempool(&self) -> Arc<Mempool> {
        Arc::clone(&self.mempool)
    }

    fn get_db(&self) -> Arc<DB> {
        Arc::clone(&self.db)
    }
}
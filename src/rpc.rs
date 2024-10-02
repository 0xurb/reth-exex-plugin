use std::path::PathBuf;

use jsonrpsee::core::RpcResult;
use tokio::sync::oneshot;

/// RPC response sender representation
pub type ResponseTx<T> = oneshot::Sender<RpcResult<T>>;

#[derive(Debug)]
pub enum RpcRequest {
    ListPlugins { tx: ResponseTx<Vec<&'static str>> },
    LoadPlugin { plugin_path: PathBuf, tx: ResponseTx<()> },
    UnloadPlugin { id: &'static str, tx: ResponseTx<()> },
}

#[macro_export]
macro_rules! format_rpc_err {
    ($($arg:tt)*) => {
        jsonrpsee::types::error::ErrorObject::owned(jsonrpsee::types::error::INTERNAL_ERROR_CODE, format!($($arg)*), None::<()>)
    };
}

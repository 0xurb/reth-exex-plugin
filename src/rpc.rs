use std::path::PathBuf;

use futures::future::BoxFuture;
use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::{error::INTERNAL_ERROR_CODE, ErrorObjectOwned as RpcError},
    RpcModule,
};
use tokio::sync::{mpsc, oneshot};

use crate::sender::Sender;

/// RPC response sender representation
pub type ResponseTx<T> = oneshot::Sender<RpcResult<T>>;

#[derive(Debug)]
pub enum RpcRequest {
    ListPlugins { tx: ResponseTx<Vec<String>> },
    LoadPlugin { plugin_path: PathBuf, tx: ResponseTx<String> },
    UnloadPlugin { id: String, tx: ResponseTx<()> },
}

#[rpc(server, namespace = "exex")]
trait ExExRpcPluginApi {
    /// Returns a list of all presented ExEx plugin ids.
    #[method(name = "listPlugins")]
    async fn list_plugins(&self) -> RpcResult<Vec<String>>;

    /// Loads ExEx plugin to the node and initializes it.
    ///
    /// Returns an ExEx plugin id.
    #[method(name = "loadPlugin")]
    async fn load_plugin(&self, plugin_path: PathBuf) -> RpcResult<String>;

    /// Unloads ExEx plugin from the node.
    #[method(name = "unloadPlugin")]
    async fn unload_plugin(&self, id: String) -> RpcResult<()>;
}

/// ExEx manager RPC module
#[derive(Debug)]
pub struct ExExPluginRpc {
    /// Request sender to ExEx plugin [manager](`crate::manager::ExExManager`).
    pub tx: Sender<RpcRequest>,
}

impl ExExPluginRpc {
    pub fn new(tx: mpsc::UnboundedSender<RpcRequest>) -> Self {
        ExExPluginRpc { tx: Sender::new(tx) }
    }

    /// Wrapper for [ExExRpcPluginApi] RPC server to [RpcModule].
    pub fn rpc_module(tx: mpsc::UnboundedSender<RpcRequest>) -> RpcModule<Self> {
        Self::new(tx).into_rpc()
    }
}

impl ExExRpcPluginApiServer for ExExPluginRpc {
    #[doc = " Returns a list of all presented ExEx plugin ids."]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn list_plugins<'a: 'b, 'b>(&'a self) -> BoxFuture<'b, RpcResult<Vec<String>>> {
        Box::pin(async move {
            let (tx, rx) = oneshot::channel();
            self.tx.send(RpcRequest::ListPlugins { tx });
            process_request_rx(rx).await
        })
    }

    #[doc = " Loads ExEx plugin to the node and initializes it."]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn load_plugin<'a: 'b, 'b>(&'a self, plugin_path: PathBuf) -> BoxFuture<'b, RpcResult<String>> {
        Box::pin(async move {
            let (tx, rx) = oneshot::channel();
            self.tx.send(RpcRequest::LoadPlugin { plugin_path, tx });
            process_request_rx(rx).await
        })
    }

    #[doc = " Unloads ExEx plugin from the node."]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn unload_plugin<'a: 'b, 'b>(&'a self, id: String) -> BoxFuture<'b, RpcResult<()>> {
        Box::pin(async move {
            let (tx, rx) = oneshot::channel();
            self.tx.send(RpcRequest::UnloadPlugin { id, tx });
            process_request_rx(rx).await
        })
    }
}

/// Helper to process response from polled [`oneshot::Receiver`]
async fn process_request_rx<T>(rx: oneshot::Receiver<RpcResult<T>>) -> RpcResult<T> {
    rx.await.map_err(|_| {
        RpcError::owned(
            INTERNAL_ERROR_CODE,
            "ExEx plugin manager oneshot channel rx was dropped.",
            None::<()>,
        )
    })?
}

#[macro_export]
macro_rules! format_rpc_err {
    ($($arg:tt)*) => {
        jsonrpsee::types::error::ErrorObject::owned(jsonrpsee::types::error::INTERNAL_ERROR_CODE, format!($($arg)*), None::<()>)
    };
}

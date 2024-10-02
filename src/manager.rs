//! [`ExExPlugin`] manager

use std::collections::HashSet;

use tokio::sync::mpsc;

use reth_exex::{ExExContext, ExExEvent, ExExNotification};
use reth_node_api::FullNodeComponents;
use reth_tracing::tracing::{debug, error, info, trace};

use crate::{plugin::LoadedExExPlugin, rpc::RpcRequest};

/// The `ExEx` plugins manager.
///
/// Dynamically loads and unloads ExEx [plugins](`super::ExExPlugin`).
#[non_exhaustive]
pub struct ExExPluginManager<Node: FullNodeComponents> {
    /// This `ExEx` context
    ctx: ExExContext<Node>,
    /// Custom extended RPC [message](`RpcRequest`) receiver.
    rpc_request_recv: mpsc::UnboundedReceiver<RpcRequest>,
    /// A list of loaded plugins.
    plugins: HashSet<LoadedExExPlugin>,
}

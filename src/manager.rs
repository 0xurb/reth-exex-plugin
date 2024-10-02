//! [`ExExPlugin`] manager

use std::{collections::HashSet, path::Path, sync::Arc};

use eyre::Result;
use futures::StreamExt;
use libloading::{Library, Symbol};
use tokio::sync::mpsc;

use reth_exex::{ExExContext, ExExEvent, ExExNotification};
use reth_node_api::FullNodeComponents;
use reth_tracing::tracing::{debug, error, info, trace};

use crate::{
    format_rpc_err,
    plugin::{LoadedExExPlugin, EXEX_MANAGER_CONSTRUCTOR_FN_NAME},
    rpc::RpcRequest,
    ExExPlugin,
};

/// Reserved ID for ExEx plugins manager.
const EXEX_MANAGER_ID: &str = "ExExManager";

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

impl<Node: FullNodeComponents> ExExPluginManager<Node> {
    pub fn new(
        ctx: ExExContext<Node>,
        rpc_request_recv: mpsc::UnboundedReceiver<RpcRequest>,
    ) -> Self {
        Self { ctx, rpc_request_recv, plugins: HashSet::default() }
    }

    /// Start a manager
    pub async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                // handle `ExExNotification` on list of loaded plugins
                Some(notification) = self.ctx.notifications.next() => {
                    self.handle_notification(notification).await?
                }
                // handle RPC request to operate with plugins or load them
                Some(req) = self.rpc_request_recv.recv() => {
                    self.handle_rpc_request(req).await
                },
            }
        }
    }

    async fn handle_notification(&mut self, notification: ExExNotification) -> Result<()> {
        for plugin in self.plugins.iter() {
            if let Err(err) = plugin.handle_notification(&notification).await {
                error!(id = %plugin.id(), %err, "failed to process notification")
            }
            info!(id = %plugin.id(), "Handled notification");
        }

        if let Some(tip) = notification.committed_chain().map(|chain| chain.tip().num_hash_slow()) {
            self.ctx.events.send(ExExEvent::FinishedHeight(tip))?;
            info!(?tip, "Handled notification");
        }

        Ok(())
    }

    #[allow(unused_must_use)] // for oneshot send error
    async fn handle_rpc_request(&mut self, req: RpcRequest) {
        match req {
            RpcRequest::ListPlugins { tx } => {
                let res = Ok(self.plugins());
                tx.send(res).inspect_err(|err| error!("failed to send response: {err:?}"));
            }
            RpcRequest::LoadPlugin { plugin_path, tx } => {
                let res = unsafe { self.load_plugin(plugin_path) }
                    .await
                    .map_err(|err| format_rpc_err!("failed to load exex plugin: {err:?}"));
                tx.send(res).inspect_err(|err| error!("failed to send response: {err:?}"));
            }
            RpcRequest::UnloadPlugin { id, tx } => {
                let res = self
                    .unload_plugin(id)
                    .map_err(|err| format_rpc_err!("failed to unload exex plugin: {err:?}"));
                tx.send(res).inspect_err(|err| error!("failed to send response: {err:?}"));
            }
        }
    }

    /// Returns a list of all plugin's ids.
    pub fn plugins(&self) -> Vec<&'static str> {
        self.plugins.iter().map(|plugin| plugin.id()).collect()
    }

    /// Load the ExEx [plugin](`super::ExExPlugin`) from a given path.
    ///
    /// # Safety
    ///
    /// The  [plugin](`super::ExExPlugin`) implementing library **must** contain a function with
    /// name [`EXEX_MANAGER_CONSTRUCTOR_FN_NAME`]. Otherwise, behavior is undefined.
    /// See also [`libloading::Library::get`] for more information on what
    /// restrictions apply to [`EXEX_MANAGER_CONSTRUCTOR_FN_NAME`].
    pub async unsafe fn load_plugin<P: AsRef<Path>>(&mut self, plugin_path: P) -> Result<()> {
        type ExExPluginCreate = unsafe fn() -> *mut dyn ExExPlugin;

        let lib = Library::new(plugin_path.as_ref())
            .map_err(|err| eyre::format_err!("Failed to find & load exex plugin: {err:?}"))?;
        let constructor: Symbol<'_, ExExPluginCreate> =
            lib.get(EXEX_MANAGER_CONSTRUCTOR_FN_NAME).map_err(|_| {
                eyre::format_err!(
                    "The `__create_exex_plugin` symbol wasn't found on exex plugin library."
                )
            })?;

        let raw_plugin_ptr = constructor();
        let mut plugin: Box<dyn ExExPlugin> = Box::from_raw(raw_plugin_ptr);
        let id = plugin.id();

        self.validate_plugin(id)?;

        trace!(id=%id, action="on_load", "calling");
        plugin.on_load().await?;

        self.plugins.insert(LoadedExExPlugin { plugin, lib: Arc::new(lib) });

        debug!(id=%id, action="load", "ExEx plugin was loaded succesfully");

        Ok(())
    }

    /// Unload the ExEx [plugin](`super::ExExPlugin`) by the given plugin id, if one exists on
    /// manager.
    pub fn unload_plugin(&mut self, id: &'static str) -> Result<()> {
        debug!(id=%id, action="ExExPluginManager::unload_plugin", "unloading an ExEx plugin");

        if let Some(mut plugin) = self.plugins.take(id) {
            trace!(id=%id, action="ExExPlugin::on_unload", "calling");
            plugin.on_unload()?;

            if Arc::strong_count(&plugin.lib) == 1 {
                trace!(id=%id, action="ExExPlugin::on_unload", "closing library");

                // Drop goes in declaration order of fields
                // So, we can assume that plugin's box drops first.
                // We don't need to call close method manually, just drop it.
                drop(plugin);
            }
        }

        debug!(id=%id, action="unload", "ExEx plugin was unloaded succesfully");

        Ok(())
    }

    /// Unload all ExEx [plugins](`super::ExExPlugin`) exists on manager.
    pub fn unload_all(&mut self) {
        info!("Start unload all ExEx plugins");

        let unload_res: Result<()> =
            self.plugins().iter().try_for_each(|name| self.unload_plugin(name));
        if let Err(err) = unload_res {
            error!(err=%err, "Error on unload plugins")
        }
    }

    /// Validates [plugin](`super::ExExPlugin`) to being:
    ///
    /// - not presented on manager (TODO: ability to replace it)
    /// - [id](`super::ExExPlugin::id`) is not equal to [`EXEX_MANAGER_ID`]
    #[inline]
    fn validate_plugin(&self, id: &'static str) -> Result<()> {
        if self.plugins.contains(id) {
            eyre::bail!("Plugin with id: `{id:?}` is already presented on manager.");
        }

        if id == EXEX_MANAGER_ID {
            eyre::bail!(
                "`{EXEX_MANAGER_ID}` is reserved id for manager. Choose another id for plugin."
            );
        }

        Ok(())
    }
}

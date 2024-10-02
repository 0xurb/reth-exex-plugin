//! ExEx plugin manager runner.

use tokio::sync::mpsc;

use reth_node_ethereum::EthereumNode;

use reth_exex_plugin::{ExExPluginManager, ExExPluginRpc, ExExRpcPluginApiServer, EXEX_MANAGER_ID};

fn main() -> eyre::Result<()> {
    reth::cli::Cli::parse_args().run(|builder, _| async move {
        // communication between manager & rpc module
        let (tx, rx) = mpsc::unbounded_channel();

        let handle = builder
            .node(EthereumNode::default())
            .extend_rpc_modules(move |ctx| {
                ctx.modules.merge_configured(ExExPluginRpc::new(tx).into_rpc())?;
                Ok(())
            })
            .install_exex(EXEX_MANAGER_ID, |ctx| async move {
                Ok(ExExPluginManager::new(ctx, rx).run())
            })
            .launch()
            .await?;

        handle.wait_for_node_exit().await
    })
}

use std::{future::Future, io, path::Path, pin::Pin};

use jsonrpsee::types::{error::INTERNAL_ERROR_CODE, ErrorObjectOwned as RpcError};
use reth::{
    chainspec::Head,
    primitives::BlockNumHash,
    providers::{Chain, ExecutionOutcome},
};
use reth_exex_plugin::{ExExPluginManager, RpcRequest};
use reth_exex_test_utils::{test_exex_context, Adapter, PollOnce, TestExExHandle};

use reth_node_api::FullNodeComponents;
use reth_tracing::init_test_tracing;
use tokio::sync::{mpsc, oneshot};

const MINIMAL_PLUGIN_PATH: &'static str = "examples/minimal/target/release/libminimal.dylib";
const MINIMAL_PLUGIN_DUMMY_STORAGE_PATH: &'static str =
    "examples/minimal/assets/notifications.json";

/// Just a test context
struct ExExPluginManagerContext<Node: FullNodeComponents> {
    head: Head,
    exex_handle: Option<TestExExHandle>,
    plugin_manager: ExExPluginManager<Node>,
}

impl ExExPluginManagerContext<Adapter> {
    async fn new(rpc_request_recv: mpsc::UnboundedReceiver<RpcRequest>) -> eyre::Result<Self> {
        // Initialize a test Execution Extension context with all dependencies
        let (exex_ctx, exex_handle) = test_exex_context().await?;
        // Save the current head of the chain to check the finished height against it later
        let head = exex_ctx.head;
        // Initialize the Execution Extension plugin manager
        let plugin_manager = ExExPluginManager::new(exex_ctx, rpc_request_recv);
        Ok(Self { head, exex_handle: Some(exex_handle), plugin_manager })
    }

    fn plugin_exex_fut(self) -> Pin<Box<dyn Future<Output = eyre::Result<()>> + Send>> {
        Box::pin(self.plugin_manager.run())
    }
}

/// Helper to check a dummy JSON minimal plugin storage
fn is_file_empty<P: AsRef<Path>>(path: P) -> io::Result<bool> {
    let metadata = std::fs::metadata(&path)?;
    Ok(metadata.len() == 0)
}

#[tokio::test]
async fn plugin_manager_receive_notifications() -> eyre::Result<()> {
    // RPC mocked channel
    let (_rpc_request_tx, rpc_request_rx) = mpsc::unbounded_channel();

    // Initialize a test Execution Extension context with all dependencies
    let mut ctx = ExExPluginManagerContext::new(rpc_request_rx).await?;
    let head = ctx.head;
    let mut exex_handle = std::mem::take(&mut ctx.exex_handle).unwrap();

    // Send a notification to the Execution Extension that the chain has been committed
    let genesis = exex_handle.genesis.clone();
    exex_handle
        .send_notification_chain_committed(Chain::from_block(
            genesis,
            ExecutionOutcome::default(),
            None,
        ))
        .await?;

    exex_handle.assert_events_empty();

    ctx.plugin_exex_fut().poll_once().await?;

    exex_handle
        .assert_event_finished_height(BlockNumHash { number: head.number, hash: head.hash })?;

    Ok(())
}

#[tokio::test]
async fn should_exec_minimal_plugin() -> eyre::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    init_test_tracing();

    // RPC mocked channel without module just to test requests
    let (rpc_request_tx, rpc_request_rx) = mpsc::unbounded_channel();
    assert!(
        is_file_empty(MINIMAL_PLUGIN_DUMMY_STORAGE_PATH)?,
        "For test JSON storage of minimal plugin must be empty"
    );

    // Initialize a test Execution Extension context with all dependencies
    let mut ctx = ExExPluginManagerContext::new(rpc_request_rx).await?;
    let head = ctx.head;
    let mut exex_handle = std::mem::take(&mut ctx.exex_handle).unwrap();

    // Send a notification to the Execution Extension that the chain has been committed
    let genesis = exex_handle.genesis.clone();

    let mut plugin_exex_fut = ctx.plugin_exex_fut();

    // Load a plugin
    let (tx, rx) = oneshot::channel();
    let load_plugin_req = RpcRequest::LoadPlugin { plugin_path: MINIMAL_PLUGIN_PATH.into(), tx };
    let _ = rpc_request_tx.send(load_plugin_req);
    // Poll the Execution Extension once to process incoming notifications or RPC requests
    plugin_exex_fut.poll_once().await?;
    assert_eq!(rx.await??.as_str(), "MinimalExEx");

    // Check a plugin list contains element
    let (tx, rx) = oneshot::channel();
    let list_plugins_req = RpcRequest::ListPlugins { tx };
    let _ = rpc_request_tx.send(list_plugins_req);
    // Poll the Execution Extension once to process incoming notifications or RPC requests
    plugin_exex_fut.poll_once().await?;
    assert_eq!(rx.await??, vec!["MinimalExEx"], "List of plugins must contain a plugin name");

    // Load the same plugin - error
    let (tx, rx) = oneshot::channel();
    let load_plugin_req = RpcRequest::LoadPlugin { plugin_path: MINIMAL_PLUGIN_PATH.into(), tx };
    let _ = rpc_request_tx.send(load_plugin_req);
    // Poll the Execution Extension once to process incoming notifications or RPC requests
    plugin_exex_fut.poll_once().await?;
    let err = rx.await?.err().expect("expect load already presented plugin error");
    assert_eq!(err.code(), INTERNAL_ERROR_CODE);
    dbg!(&err);
    assert!(err.message().contains("failed to load exex plugin: Plugin with id: `\"MinimalExEx\"` is already presented on manager."));

    exex_handle
        .send_notification_chain_committed(Chain::from_block(
            genesis,
            ExecutionOutcome::default(),
            None,
        ))
        .await?;

    // Receive a notification
    // Check that the Execution Extension did not emit any events until we polled it
    exex_handle.assert_events_empty();
    // Poll the Execution Extension once to process incoming notifications or RPC requests
    plugin_exex_fut.poll_once().await?;
    // Check that dummy json storage is filled with it - TODO
    assert!(
        !is_file_empty(MINIMAL_PLUGIN_DUMMY_STORAGE_PATH)?,
        "JSON storage of minimal plugin must be filled with notification"
    );

    // Unload a plugin
    let (tx, rx) = oneshot::channel();
    let unload_plugin_req = RpcRequest::UnloadPlugin { id: "MinimalExEx".to_owned(), tx };
    let _ = rpc_request_tx.send(unload_plugin_req);
    // Poll the Execution Extension once to process incoming notifications or RPC requests
    plugin_exex_fut.poll_once().await?;
    rx.await??;

    // Check a plugin list is empty
    let (tx, rx) = oneshot::channel();
    let list_plugins_req = RpcRequest::ListPlugins { tx };
    let _ = rpc_request_tx.send(list_plugins_req);
    // Poll the Execution Extension once to process incoming notifications or RPC requests
    plugin_exex_fut.poll_once().await?;
    assert!(rx.await??.is_empty());

    // Check that the Execution Extension emitted a `FinishedHeight` event with the correct
    // height
    exex_handle
        .assert_event_finished_height(BlockNumHash { number: head.number, hash: head.hash })?;

    Ok(())
}

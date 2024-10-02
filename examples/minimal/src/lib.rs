//! ExEx plugin example implementation.
//! 
//! Simply takes a notification's chain kind & range of block numbers
//!     and store them to `OUT_PATH` json file, if it was either revert or commit.

use std::{future::Future, pin::Pin};

use eyre::Result;
use serde::Serialize;
use reth_exex_plugin::{ExExNotification, ExExPlugin};

const OUT_PATH: &'static str = "../assets/notifications.json";

#[derive(Serialize)]
enum ProcessedExExNotification {
    Commit { from: u64, to: u64 },
    Revert { from: u64, to: u64 }
}

#[derive(Debug, Default)]
pub(crate) struct MinimalExEx;

impl ExExPlugin for MinimalExEx {
    fn id(&self) -> &'static str {
        "MinimalExEx"
    }

    /// Example usage of loading hook
    fn on_load<'a: 'b, 'b>(&'a mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'b>> {
        Box::pin(async move { Ok(()) })
    }

    /// Example usage of unloading hook
    fn on_unload(&mut self) -> Result<()> {
        Ok(())
    }

    /// Example usage of [notification](`ExExNotification`) handler
    /// 
    /// Simply takes a notification's chain kind & range of block numbers
    ///     and store them to `OUT_PATH` json file, if it was either revert or commit.
    fn handle_notification<'a: 'b, 'b>(
        &'a self,
        notification: &'a ExExNotification,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'b>> {
        Box::pin(async move {
            match notification {
                ExExNotification::ChainCommitted { new } => {
                    // received commit
                    let range = new.range();
                    write_notification(ProcessedExExNotification::Commit {from: *range.start(), to: *range.end() })
                }
                ExExNotification::ChainReverted { old } => {
                    // received revert
                    let range = old.range();
                    write_notification(ProcessedExExNotification::Revert {from: *range.start(), to: *range.end() })
                }
                _ => Ok(())
            }
        })
    }
}

/// Writes a given [ProcessedExExNotification] in the [`OUT_PATH`]
fn write_notification(notification: ProcessedExExNotification) -> Result<()> {
    std::fs::write(OUT_PATH, serde_json::to_string_pretty(&notification)?)
        .map_err(Into::into)
}

reth_exex_plugin::declare_exex_plugin!(MinimalExEx);

/// Plugin constructor
///
/// # Safety
///
/// See [`ExExPlugin`] loading on [`reth_exex_plugin`] crate.
/// Especeally, a manager declaration with method for plugin load.
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn __create_exex_plugin() -> *mut dyn ExExPlugin {
    let plugin = MinimalExEx;
    let plugin: Box<dyn ExExPlugin> = Box::new(plugin);
    Box::into_raw(plugin)
}

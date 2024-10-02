//! ExEx plugin core dependencies & type-safe abstractions
//!     which allows to build plugins by implementing [`ExExPlugin`] trait
//!     on the dynamic libraries.

mod plugin;
pub use plugin::ExExPlugin;

mod manager;
pub use manager::{ExExPluginManager, EXEX_MANAGER_ID};

mod rpc;
pub use rpc::{ExExPluginRpc, ExExRpcPluginApiServer};

mod sender;

/// re-export for [`ExExNotification`] type
pub use reth_exex::ExExNotification;

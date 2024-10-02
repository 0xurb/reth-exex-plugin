//! ExEx plugin core dependencies & type-safe abstractions
//!     which allows to build plugins by implementing [`ExExPlugin`] trait
//!     on the dynamic libraries.

#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![allow(missing_docs)]

mod plugin;
pub use plugin::ExExPlugin;

mod manager;

mod rpc;

mod sender;

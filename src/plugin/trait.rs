//! ExEx plugin interface

use std::{borrow::Borrow, fmt::Debug, future::Future, hash::Hash, pin::Pin};

use eyre::Result;

use reth_exex::ExExNotification;

/// Required name of the plugin contrusctor function.
pub const EXEX_MANAGER_CONSTRUCTOR_FN_NAME: &[u8] = b"__create_exex_plugin";

/// ExEx plugin trait.
/// # Example - Declare ExEx Plugin
/// ```rust
/// use std::{future::Future, pin::Pin};
///
/// use eyre::Result;
///
/// use reth_exex::ExExNotification;
/// use reth_exex_plugin::ExExPlugin;
///
/// #[derive(Debug, Default)]
/// struct MinimalExEx;
///
/// impl ExExPlugin for MinimalExEx {
///     fn id(&self) -> &'static str {
///         "MinimalExEx"
///     }
///
///     fn handle_notification<'a: 'b, 'b>(
///         &'a self,
///         notification: &'a ExExNotification,
///     ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'b>> {
///         Box::pin(async { Ok(()) })
///     }
/// }
///
/// reth_exex_plugin::declare_exex_plugin!(MinimalExEx);
/// ```
pub trait ExExPlugin: Debug + Send + Sync + 'static {
    fn id(&self) -> &'static str;

    /// Plugin's semantic version.
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// A hook fired immediately after the plugin is loaded by the system.
    ///
    /// Used for any initialization logic.
    fn on_load<'a: 'b, 'b>(&'a mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'b>> {
        Box::pin(async { Ok(()) })
    }

    /// A callback fired immediately before the plugin is unloaded.
    ///
    /// Used for doing any cleanup before unload.
    fn on_unload(&mut self) -> Result<()> {
        Ok(())
    }

    /// Method to handle received ExEx [notification](ExExNotification).
    fn handle_notification<'a: 'b, 'b>(
        &'a self,
        notification: &'a ExExNotification,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'b>>;
}

impl Hash for dyn ExExPlugin + '_ {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Borrow<str> for Box<dyn ExExPlugin> {
    fn borrow(&self) -> &str {
        self.id()
    }
}

/// Declare an ExEx plugin type and its constructor.
///
/// # Notes
///
/// This works by automatically generating an `extern "C"` function with a
/// pre-defined signature and symbol name. Therefore you will only be able to
/// declare one plugin per library.
#[macro_export]
macro_rules! declare_exex_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn _create_exex_plugin() -> *mut dyn $crate::ExExPlugin {
            let boxed: Box<dyn $crate::ExExPlugin> = Box::new(<$plugin_type>::default());
            Box::into_raw(boxed)
        }
    };

    ($plugin_type:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _create_exex_plugin() -> *mut dyn $crate::ExExPlugin {
            // make sure the constructor is the correct type.
            let constructor: fn() -> $plugin_type = $constructor;

            let object = constructor();
            let boxed: Box<dyn $crate::ExExPlugin> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}

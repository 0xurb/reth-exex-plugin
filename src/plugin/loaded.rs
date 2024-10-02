//! A loaded ExEx plugin

use std::{
    borrow::Borrow,
    hash::Hash,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use eyre::Result;
use libloading::Library;

use reth_exex::ExExNotification;

use super::ExExPlugin;

#[derive(Debug)]
pub(crate) struct LoadedExExPlugin {
    pub(crate) plugin: Box<dyn ExExPlugin>,
    pub(crate) lib: Arc<Library>,
}

impl Borrow<str> for LoadedExExPlugin {
    fn borrow(&self) -> &str {
        self.id()
    }
}

impl PartialEq for LoadedExExPlugin {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for LoadedExExPlugin {}

impl Hash for LoadedExExPlugin {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.plugin.hash(state);
    }
}

impl Deref for LoadedExExPlugin {
    type Target = Box<dyn ExExPlugin>;

    fn deref(&self) -> &Self::Target {
        &self.plugin
    }
}

impl DerefMut for LoadedExExPlugin {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.plugin
    }
}

impl LoadedExExPlugin {
    #[inline(always)]
    pub(crate) fn id(&self) -> &'static str {
        self.plugin.id()
    }

    pub(crate) async fn handle_notification(&self, notification: &ExExNotification) -> Result<()> {
        self.plugin.handle_notification(notification).await
    }
}

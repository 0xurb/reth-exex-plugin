//! Wrapper around [mpsc::UnboundedSender]
//! with a `receiver_dropped` flag for keeping track of channel.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tokio::sync::mpsc;

use reth_tracing::tracing::warn;

#[derive(Debug, Clone)]
pub struct Sender<T: Send> {
    receiver_dropped: Arc<AtomicBool>,
    tx: mpsc::UnboundedSender<T>,
}

impl<T: Send> Sender<T> {
    pub fn new(tx: mpsc::UnboundedSender<T>) -> Self {
        Self { receiver_dropped: Arc::new(AtomicBool::new(false)), tx }
    }
}

impl<T: Send> Sender<T> {
    pub fn send(&self, msg: T) {
        if self.receiver_dropped() {
            return;
        }

        if let Err(e) = self.tx.send(msg) {
            warn!("[Sender] Receiver was dropped on error while send. Error: {e}");
            self.receiver_dropped.store(true, Ordering::SeqCst);
        }
    }

    pub fn send_many(&self, msgs: Vec<T>) {
        if self.receiver_dropped() {
            return;
        }

        msgs.into_iter().for_each(|msg| {
            let _ = self.tx.send(msg);
        })
    }

    fn receiver_dropped(&self) -> bool {
        self.receiver_dropped.load(Ordering::SeqCst)
    }
}

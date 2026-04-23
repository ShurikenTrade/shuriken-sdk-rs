use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;
use tokio::sync::mpsc;

use crate::error::ShurikenError;

pub struct Subscription<T> {
    pub(crate) rx: mpsc::UnboundedReceiver<Result<T, ShurikenError>>,
    pub(crate) id: usize,
    pub(crate) unsub_tx: mpsc::UnboundedSender<usize>,
}

impl<T> Stream for Subscription<T> {
    type Item = Result<T, ShurikenError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

impl<T> Drop for Subscription<T> {
    fn drop(&mut self) {
        let _ = self.unsub_tx.send(self.id);
    }
}

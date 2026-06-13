use std::sync::mpsc::{channel, Receiver, TryRecvError};

pub struct FutureResult<T> {
    pub(crate) channel: Receiver<T>,
}

impl<T> FutureResult<T> {
    /// Wait until host sends the result back.
    pub fn blocking_recv(&self) -> T {
        self.channel.recv().expect("Host dropped the sender without responding")
    }

    /// Extract the inner mpsc channel for more control.
    pub fn extract_inner(self) -> Receiver<T> {
        self.channel
    }
}

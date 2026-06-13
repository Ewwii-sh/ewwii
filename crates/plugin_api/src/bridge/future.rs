use std::sync::mpsc::{channel, Receiver, RecvError};

pub struct FutureResult<T> {
    pub(crate) channel: Receiver<T>,
}

impl<T: Send + 'static> FutureResult<T> {
    /// Resolve the future in another thread.
    pub fn resolve(self) -> Result<T, RecvError> {
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let result = self.channel.recv();
            let _ = tx.send(result);
        });
        rx.recv().and_then(|inner_result| inner_result)
    }

    /// Extract the inner mpsc channel for more control.
    ///
    /// WARNING: Always resolve in a separate thread
    /// as blocking main thread will resolve in a deadlock.
    pub fn extract_inner(self) -> Receiver<T> {
        self.channel
    }
}

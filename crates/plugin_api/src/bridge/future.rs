use std::sync::mpsc::{Receiver, RecvError};

pub struct FutureResult<T> {
    pub(crate) channel: Receiver<T>,
}

impl<T: Send + 'static> FutureResult<T> {
    /// Resolve the future by blocking current thread.
    ///
    /// # Safety
    ///
    /// Resolve in another thread with something like [`std::thread::spawn`]
    /// to avoid dead blocking the main thread.
    pub fn resolve(self) -> Result<T, RecvError> {
        self.channel.recv()
    }

    /// Non-blocking resolution: fires a callback when the result is ready.
    pub fn resolve_async<F>(self, callback: F)
    where
        F: FnOnce(Result<T, RecvError>) + Send + 'static,
    {
        std::thread::spawn(move || {
            let result = self.channel.recv();
            callback(result);
        });
    }

    /// Extract the inner mpsc channel for more control.
    ///
    /// WARNING: Always resolve in a separate thread
    /// as blocking main thread will resolve in a deadlock.
    pub fn extract_inner(self) -> Receiver<T> {
        self.channel
    }
}

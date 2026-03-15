use bytes::Bytes;
use tokio::sync::mpsc;

pub mod ble;

/// A byte-stream connection to a MeshCore device.
///
/// `connect()` returns the receive half of an mpsc channel. The transport
/// owns the send half and sends incoming bytes into it from a background task.
/// Dropped connections cause `rx.recv()` to return `None`.
pub trait Transport: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    fn connect(&mut self) -> impl Future<Output = Result<mpsc::Receiver<Bytes>, Self::Error>>;
    fn send(&self, data: &[u8]) -> impl Future<Output = Result<(), Self::Error>>;
    fn disconnect(&mut self) -> impl Future<Output = Result<(), Self::Error>>;
}

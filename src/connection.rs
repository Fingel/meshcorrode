use std::sync::Arc;
use std::time::Duration;

use log::warn;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc;

use crate::commands::Command;
use crate::error::Error;
use crate::event::Event;
use crate::event_bus::EventBus;
use crate::proto::parser::Parser;
use crate::transport::Transport;

pub struct Connection<T: Transport> {
    transport: T,
    event_bus: Arc<EventBus>,
}

impl<T: Transport> Connection<T> {
    pub async fn connect(mut transport: T) -> Result<Self, T::Error> {
        let mut rx = transport.connect().await?;
        let event_bus = Arc::new(EventBus::new());
        let bus = event_bus.clone();

        tokio::spawn(async move {
            let mut parser = Parser::new();
            while let Some(bytes) = rx.recv().await {
                match parser.parse_packet(&bytes) {
                    Ok(Some(event)) => bus.publish(event),
                    Ok(None) => {}
                    Err(e) => warn!("failed to parse packet: {e}"),
                }
            }
        });

        Ok(Self {
            transport,
            event_bus,
        })
    }

    /// Execute a one-shot command and wait for a matching response (or error) with a timeout.
    pub async fn execute<C: Command + Send>(
        &self,
        cmd: C,
        timeout: Duration,
    ) -> Result<Event, Error> {
        let mut rx = self.event_bus.subscribe();

        self.transport
            .send(&cmd.encode())
            .await
            .map_err(|e| Error::Transport(Box::new(e)))?;

        tokio::time::timeout(timeout, async move {
            loop {
                match rx.recv().await {
                    Ok(event) if cmd.is_response(&event) => return Ok(event),
                    Ok(Event::Error(e)) => return Err(Error::Device(e)),
                    Ok(_) => continue,
                    Err(RecvError::Lagged(n)) => warn!("event bus lagged by {n} events"),
                    Err(RecvError::Closed) => return Err(Error::Disconnected),
                }
            }
        })
        .await
        .map_err(|_| Error::Timeout)?
    }

    /// Subscribe to events matching a predicate function.
    ///
    /// Returns a receiver that only returns events matching the
    /// supplied predicate. This prevents backpressure
    /// from building up in the event bus from events that the caller doesn't care about.
    /// Note that device errors are ignored - these could be from other commands.
    /// If the caller wants to be notified of device errors, they should add them
    /// to the predicate.
    pub fn subscribe<F>(&self, predicate: F) -> mpsc::Receiver<Event>
    where
        F: Fn(&Event) -> bool + Send + 'static,
    {
        let mut broadcast_rx = self.event_bus.subscribe();
        let (tx, rx) = mpsc::channel(16);

        tokio::spawn(async move {
            loop {
                match broadcast_rx.recv().await {
                    Ok(event) if predicate(&event) => {
                        if tx.send(event).await.is_err() {
                            break;
                        }
                    }
                    Ok(_) => continue,
                    Err(RecvError::Lagged(n)) => warn!("subscription lagged by {n} events"),
                    Err(RecvError::Closed) => break,
                }
            }
        });

        rx
    }

    /// Subscribe to all events without filtering.
    pub fn subscribe_all(&self) -> tokio::sync::broadcast::Receiver<Event> {
        self.event_bus.subscribe()
    }
}

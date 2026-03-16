use std::sync::Arc;
use std::time::Duration;

use log::warn;
use tokio::sync::broadcast::error::RecvError;

use crate::commands::Command;
use crate::error::Error;
use crate::event::Event;
use crate::event_bus::EventBus;
use crate::proto::parser::parse_packet;
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
            while let Some(bytes) = rx.recv().await {
                match parse_packet(&bytes) {
                    Ok(event) => bus.publish(event),
                    Err(e) => warn!("failed to parse packet: {e}"),
                }
            }
        });

        Ok(Self {
            transport,
            event_bus,
        })
    }

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

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Event> {
        self.event_bus.subscribe()
    }
}

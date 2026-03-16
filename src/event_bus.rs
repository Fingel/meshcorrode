use tokio::sync::broadcast;

use crate::event::Event;

pub struct EventBus {
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(64);
        Self { sender }
    }

    pub fn publish(&self, event: Event) {
        // A send error means no active receivers, so we can ignore the result here
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

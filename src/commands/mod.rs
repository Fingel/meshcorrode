pub mod contact;
pub mod device;

use bytes::Bytes;

use crate::event::Event;

pub trait Command {
    type Response;
    fn encode(&self) -> Bytes;
    fn extract_response(&self, event: Event) -> Option<Self::Response>;
}

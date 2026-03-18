pub mod contact;
pub mod device;

use bytes::Bytes;

use crate::event::Event;

pub trait Command {
    fn encode(&self) -> Bytes;
    fn is_response(&self, event: &Event) -> bool;
}

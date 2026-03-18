pub mod contact;
pub mod device;

use bytes::Bytes;

use crate::event::Event;

const CMD_APP_START: u8 = 1;
const CMD_GET_CONTACTS: u8 = 0x04;

pub trait Command {
    type Response;
    fn encode(&self) -> Bytes;
    fn extract_response(&self, event: Event) -> Option<Self::Response>;
}

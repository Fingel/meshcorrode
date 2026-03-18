pub mod contact;
pub mod device;
pub mod messaging;

use bytes::Bytes;

use crate::event::Event;

// https://github.com/meshcore-dev/MeshCore/blob/792f299986619aa62dad937f717d21e9d1b1882b/examples/companion_radio/MyMesh.cpp#L6
const CMD_APP_START: u8 = 1;
const CMD_SEND_TXT_MSG: u8 = 2;
const CMD_GET_CONTACTS: u8 = 0x04;

pub trait Command {
    type Response;
    fn encode(&self) -> Bytes;
    fn extract_response(&self, event: Event) -> Option<Self::Response>;
}

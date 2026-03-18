use bytes::{BufMut, Bytes, BytesMut};

use super::Command;
use crate::event::{Event, SelfInfoPayload};

const CMD_APP_START: u8 = 1;

/// APP_START command.
///
/// Wire format: [CMD_APP_START(1)] [protocol version?(1)] [reserved(6)] [client_name(variable)]
pub struct AppStart {
    pub client_name: String,
}

impl Command for AppStart {
    fn encode(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(8 + self.client_name.len());
        buf.put_u8(CMD_APP_START);
        buf.put_u8(0x03); // protocol version TODO: find documentation on this
        buf.put_bytes(0x00, 6); // reserved bytes 2–7
        buf.put(self.client_name.as_bytes());
        buf.freeze()
    }

    type Response = SelfInfoPayload;

    fn extract_response(&self, event: Event) -> Option<SelfInfoPayload> {
        match event {
            Event::SelfInfo(p) => Some(p),
            _ => None,
        }
    }
}

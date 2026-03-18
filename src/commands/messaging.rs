use bytes::{BufMut, Bytes, BytesMut};
use hex;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{CMD_SEND_TXT_MSG, Command};
use crate::event::{Event, MsgSentPayload};

#[repr(u8)]
enum MessageType {
    Text = 0,
    Command = 1,
}

pub struct Destination(Vec<u8>);

impl TryFrom<&str> for Destination {
    type Error = hex::FromHexError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(Self(hex::decode(s)?))
    }
}

impl AsRef<[u8]> for Destination {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

pub struct SendTextMessage {
    dst: Destination,
    msg: String,
    timestamp: Option<u32>,
    attempt: u8,
}

impl Command for SendTextMessage {
    fn encode(&self) -> Bytes {
        let timestamp: u32 = self.timestamp.unwrap_or(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32,
        );
        let mut buf = BytesMut::with_capacity(3 + 4 + 32 + self.msg.len());
        buf.put_u8(CMD_SEND_TXT_MSG);
        buf.put_u8(MessageType::Text as u8);
        buf.put_u8(self.attempt);
        buf.put_u32_le(timestamp);
        buf.extend_from_slice(self.dst.as_ref());
        buf.extend_from_slice(self.msg.as_bytes());
        buf.freeze()
    }

    type Response = MsgSentPayload;

    fn extract_response(&self, event: Event) -> Option<Self::Response> {
        match event {
            Event::MsgSent(payload) => Some(payload),
            _ => None,
        }
    }
}

use bytes::Bytes;

use crate::event::{ContactsPayload, Event};

use super::Command;

const CMD_GET_CONTACTS: u8 = 0x04;

#[derive(Default)]
pub struct GetContacts {
    /// Only return contacts modified after this timestamp or 0 for all contacts (default).
    pub lastmod: u32,
}

impl Command for GetContacts {
    fn encode(&self) -> Bytes {
        if self.lastmod > 0 {
            let mut buf = [0u8; 5];
            buf[0] = CMD_GET_CONTACTS;
            buf[1..].copy_from_slice(&self.lastmod.to_le_bytes());
            Bytes::copy_from_slice(&buf)
        } else {
            Bytes::from_static(&[CMD_GET_CONTACTS])
        }
    }

    type Response = ContactsPayload;

    fn extract_response(&self, event: Event) -> Option<ContactsPayload> {
        match event {
            Event::Contacts(p) => Some(p),
            _ => None,
        }
    }
}

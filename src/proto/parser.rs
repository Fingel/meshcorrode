use bytes::Buf;

use super::PacketType;
use crate::event::{
    ContactPayload, ContactType, ContactsPayload, Event, MsgSentPayload, RxLogDataPayload,
    SelfInfoPayload,
};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("packet too short: need at least {needed} bytes, got {got}")]
    TooShort { needed: usize, got: usize },
    #[error("unknown packet type: {0:#04x}")]
    UnknownPacketType(u8),
}

#[derive(Default)]
pub struct Parser {
    contact_buf: Vec<ContactPayload>,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_packet(&mut self, data: &[u8]) -> Result<Option<Event>, ParseError> {
        if data.is_empty() {
            return Err(ParseError::TooShort { needed: 1, got: 0 });
        }

        let packet_type = data[0];
        let payload = &data[1..];

        match PacketType::try_from(packet_type) {
            Ok(PacketType::SelfInfo) => parse_self_info(payload).map(|p| Some(Event::SelfInfo(p))),
            Ok(PacketType::RxLogData) => Ok(Some(Event::RxLogData(parse_rx_log_data(payload)))),
            Ok(PacketType::ContactStart) => {
                self.contact_buf.clear();
                Ok(None) // TODO maybe remove None here, to enable streaming of contacts back to the client
            }
            Ok(PacketType::Contact) => {
                self.contact_buf.push(parse_contact(payload));
                Ok(None)
            }
            Ok(PacketType::ContactEnd) => {
                let mut buf = payload;
                let lastmod = buf.get_u32_le();
                let contacts = std::mem::take(&mut self.contact_buf);
                Ok(Some(Event::Contacts(ContactsPayload { contacts, lastmod })))
            }
            Ok(PacketType::Sent) => parse_msg_sent(payload).map(|p| Some(Event::MsgSent(p))),
            Ok(PacketType::Ok) | Ok(PacketType::Error) => todo!(),
            Err(_) => Err(ParseError::UnknownPacketType(packet_type)),
        }
    }
}

// --- CONTACT ---

// CONTACT / PUSH_CODE_NEW_ADVERT
// This logic is used for parsing contacts both when explicitly requested and when we receive new adverts
// https://github.com/meshcore-dev/meshcore_py/blob/5bfe63912c6389faa072c19d2d90a2c12d23205f/src/meshcore/reader.py#L98
fn parse_contact(data: &[u8]) -> ContactPayload {
    let mut buf = data;

    let mut public_key = [0u8; 32];
    buf.copy_to_slice(&mut public_key);

    let contact_type = ContactType::try_from(buf.get_u8()).unwrap_or(ContactType::None);
    let flags = buf.get_u8();

    let plen = buf.get_u8();
    let (out_path_len, out_path_hash_mode) = if plen == 255 {
        (-1i8, -1i8)
    } else {
        ((plen & 0x3F) as i8, (plen >> 6) as i8)
    };

    let mut out_path_raw = [0u8; 64];
    buf.copy_to_slice(&mut out_path_raw);
    let out_path = out_path_raw
        .iter()
        .copied()
        .take_while(|&b| b != 0)
        .collect();

    let mut adv_name_raw = [0u8; 32];
    buf.copy_to_slice(&mut adv_name_raw);
    let adv_name = String::from_utf8_lossy(
        adv_name_raw
            .iter()
            .take_while(|&&b| b != 0)
            .copied()
            .collect::<Vec<_>>()
            .as_slice(),
    )
    .into_owned();

    let last_advert = buf.get_u32_le();
    let adv_lat = buf.get_i32_le() as f64 / 1_000_000.0;
    let adv_lon = buf.get_i32_le() as f64 / 1_000_000.0;
    let lastmod = buf.get_u32_le();

    ContactPayload {
        public_key,
        contact_type,
        flags,
        out_path_len,
        out_path_hash_mode,
        out_path,
        adv_name,
        last_advert,
        adv_lat,
        adv_lon,
        lastmod,
    }
}

// --- DEVICE ---

// RX_LOG_DATA
// https://github.com/meshcore-dev/meshcore_py/blob/5bfe63912c6389faa072c19d2d90a2c12d23205f/src/meshcore/reader.py#L577
fn parse_rx_log_data(data: &[u8]) -> RxLogDataPayload {
    let mut buf = data;
    let snr = buf.get_i8() as f32 / 4.0;
    let rssi = buf.get_i8();
    let payload = buf.to_vec();
    // TODO parse full packet: https://github.com/meshcore-dev/meshcore_py/blob/5bfe63912c6389faa072c19d2d90a2c12d23205f/src/meshcore/meshcore_parser.py#L35
    RxLogDataPayload { snr, rssi, payload }
}

// SELF_INFO
// https://github.com/meshcore-dev/meshcore_py/blob/5bfe63912c6389faa072c19d2d90a2c12d23205f/src/meshcore/reader.py#L153
fn parse_self_info(data: &[u8]) -> Result<SelfInfoPayload, ParseError> {
    let mut buf = data;

    let adv_type = buf.get_u8();
    let tx_power = buf.get_u8();
    let max_tx_power = buf.get_u8();

    let mut public_key = [0u8; 32];
    buf.copy_to_slice(&mut public_key);

    let adv_lat = buf.get_i32_le() as f64 / 1_000_000.0;
    let adv_lon = buf.get_i32_le() as f64 / 1_000_000.0;

    let multi_acks = buf.get_u8();
    let adv_loc_policy = buf.get_u8();

    let telemetry_mode = buf.get_u8();
    let telemetry_mode_env = (telemetry_mode >> 4) & 0b11;
    let telemetry_mode_loc = (telemetry_mode >> 2) & 0b11;
    let telemetry_mode_base = telemetry_mode & 0b11;

    let manual_add_contacts = buf.get_u8() != 0;

    let radio_freq = buf.get_u32_le() as f64 / 1000.0;
    let radio_bw = buf.get_u32_le() as f64 / 1000.0;
    let radio_sf = buf.get_u8();
    let radio_cr = buf.get_u8();

    let name = String::from_utf8_lossy(buf).into_owned();

    Ok(SelfInfoPayload {
        adv_type,
        tx_power,
        max_tx_power,
        public_key,
        adv_lat,
        adv_lon,
        multi_acks,
        adv_loc_policy,
        telemetry_mode_env,
        telemetry_mode_loc,
        telemetry_mode_base,
        manual_add_contacts,
        radio_freq,
        radio_bw,
        radio_sf,
        radio_cr,
        name,
    })
}

// --- MESSAGING ---

// MSG_SENT
// https://github.com/meshcore-dev/meshcore_py/blob/5bfe63912c6389faa072c19d2d90a2c12d23205f/src/meshcore/reader.py#L183
fn parse_msg_sent(data: &[u8]) -> Result<MsgSentPayload, ParseError> {
    let mut buf = data;

    let r#type = buf.get_u8(); // TODO this might be the Messaging::MessageType enum
    let mut expected_ack = [0u8; 4];
    buf.copy_to_slice(&mut expected_ack);
    let suggested_timeout = buf.get_u32_le();

    Ok(MsgSentPayload {
        r#type,
        expected_ack,
        suggested_timeout,
    })
}

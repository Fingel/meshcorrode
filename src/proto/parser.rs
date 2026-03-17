use bytes::Buf;

use super::PacketType;
use crate::event::{Event, RxLogDataPayload, SelfInfoPayload};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("packet too short: need at least {needed} bytes, got {got}")]
    TooShort { needed: usize, got: usize },
    #[error("unknown packet type: {0:#04x}")]
    UnknownPacketType(u8),
}

pub fn parse_packet(data: &[u8]) -> Result<Event, ParseError> {
    if data.is_empty() {
        return Err(ParseError::TooShort { needed: 1, got: 0 });
    }

    let packet_type = data[0];
    let payload = &data[1..];

    match PacketType::try_from(packet_type) {
        Ok(PacketType::SelfInfo) => parse_self_info(payload).map(Event::SelfInfo),
        Ok(PacketType::RxLogData) => Ok(Event::RxLogData(parse_rx_log_data(payload))),
        Ok(PacketType::Ok) | Ok(PacketType::Error) => todo!(),
        Err(_) => Err(ParseError::UnknownPacketType(packet_type)),
    }
}

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

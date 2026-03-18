use num_enum::TryFromPrimitive;

#[derive(Debug, Clone)]
pub enum Event {
    SelfInfo(SelfInfoPayload),
    Contacts(ContactsPayload),
    RxLogData(RxLogDataPayload),
    MsgSent(MsgSentPayload),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct SelfInfoPayload {
    pub adv_type: u8,
    pub tx_power: u8,
    pub max_tx_power: u8,
    pub public_key: [u8; 32],
    pub adv_lat: f64,
    pub adv_lon: f64,
    pub multi_acks: u8,
    pub adv_loc_policy: u8,
    pub telemetry_mode_env: u8,
    pub telemetry_mode_loc: u8,
    pub telemetry_mode_base: u8,
    pub manual_add_contacts: bool,
    pub radio_freq: f64,
    pub radio_bw: f64,
    pub radio_sf: u8,
    pub radio_cr: u8,
    pub name: String,
}

/// Contact node type.
/// https://github.com/meshcore-dev/meshcore_py/blob/5bfe63912c6389faa072c19d2d90a2c12d23205f/src/meshcore/meshcore_parser.py#L13
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
pub enum ContactType {
    None = 0,
    Cli = 1,
    Rep = 2,
    Room = 3,
    Sens = 4,
}

impl std::fmt::Display for ContactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContactType::None => write!(f, "NONE"),
            ContactType::Cli => write!(f, "CLI"),
            ContactType::Rep => write!(f, "REP"),
            ContactType::Room => write!(f, "ROOM"),
            ContactType::Sens => write!(f, "SENS"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContactPayload {
    pub public_key: [u8; 32],
    pub contact_type: ContactType,
    pub flags: u8,
    pub out_path_len: i8,       // -1 = flood
    pub out_path_hash_mode: i8, // -1 = flood
    pub out_path: Vec<u8>,
    pub adv_name: String,
    pub last_advert: u32,
    pub adv_lat: f64,
    pub adv_lon: f64,
    pub lastmod: u32,
}

#[derive(Debug, Clone)]
pub struct ContactsPayload {
    pub contacts: Vec<ContactPayload>,
    pub lastmod: u32,
}

#[derive(Debug, Clone)]
pub struct RxLogDataPayload {
    pub snr: f32,
    pub rssi: i8,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct MsgSentPayload {
    pub r#type: u8,
    pub expected_ack: [u8; 4],
    pub suggested_timeout: u32,
}

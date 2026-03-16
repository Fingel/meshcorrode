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

#[derive(Debug, Clone)]
pub enum Event {
    SelfInfo(SelfInfoPayload),
    Error(String),
}

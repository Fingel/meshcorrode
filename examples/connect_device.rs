use meshcorrode::{
    commands::device::app_start,
    proto::parser::parse_packet,
    transport::{
        Transport,
        ble::{BleFilter, BleTransport},
    },
};

#[tokio::main]
async fn main() {
    // Run with RUST_LOG=debug to see verbose logs
    env_logger::init();
    let mut transport = BleTransport::new(BleFilter::AnyMeshCore);
    let mut rx = transport.connect().await.unwrap();
    // Hardcoded app start bytes for now, will be commands in the future
    let app_start = app_start("meshcorrode");
    transport.send(&app_start).await.unwrap();
    if let Some(bytes) = rx.recv().await {
        let resp = parse_packet(bytes.as_ref()).unwrap();
        println!("event: {:?}", resp);
    }
}

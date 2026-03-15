use meshcorrode::transport::{
    Transport,
    ble::{BleFilter, BleTransport},
};

#[tokio::main]
async fn main() {
    let mut transport = BleTransport::new(BleFilter::AnyMeshCore);
    let mut rx = transport.connect().await.unwrap();
    // Hardcoded app start bytes for now, will be commands in the future
    let mut app_start = vec![0x01, 0x03];
    app_start.extend_from_slice(b"      mccli");
    transport.send(&app_start).await.unwrap();
    if let Some(bytes) = rx.recv().await {
        println!("appstart response: {:02x?}", bytes.as_ref());
    }
}

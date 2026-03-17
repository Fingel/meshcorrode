use std::time::Duration;

use meshcorrode::{
    commands::device::AppStart,
    connection::Connection,
    event::Event,
    transport::ble::{BleFilter, BleTransport},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Run with RUST_LOG=debug to see verbose logs
    env_logger::init();

    let transport = BleTransport::new(BleFilter::AnyMeshCore);
    let conn = Connection::connect(transport).await?;

    conn.execute(
        AppStart {
            client_name: "meshcorrode".into(),
        },
        Duration::from_secs(5),
    )
    .await?;

    println!("Listening for mesh traffic...");
    let mut rx = conn.subscribe(|e| matches!(e, Event::RxLogData(_)));

    while let Some(event) = rx.recv().await {
        if let Event::RxLogData(data) = event {
            println!(
                "rx: snr={:.1}dB rssi={}dBm payload_len={}\n{}",
                data.snr,
                data.rssi,
                data.payload.len(),
                data.payload
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            );
        }
    }

    Ok(())
}

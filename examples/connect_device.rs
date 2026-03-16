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
    let conn = Connection::connect(transport).await.unwrap();

    let event = conn
        .execute(
            AppStart {
                client_name: "meshcorrode".into(),
            },
            Duration::from_secs(5),
        )
        .await?;

    if let Event::SelfInfo(info) = event {
        println!("connected to: {}", info.name);
        println!(
            "public key:   {}",
            info.public_key.map(|b| format!("{b:02x}")).join("")
        );
    }

    Ok(())
}

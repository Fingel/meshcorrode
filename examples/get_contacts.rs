use std::time::Duration;

use meshcorrode::{
    commands::contact::GetContacts,
    connection::Connection,
    transport::ble::{BleFilter, BleTransport},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Run with RUST_LOG=debug to see verbose logs
    env_logger::init();

    let transport = BleTransport::new(BleFilter::AnyMeshCore);
    let conn = Connection::connect(transport).await?;

    println!("Fetching contacts...");
    let contacts = conn
        .execute(GetContacts::default(), Duration::from_secs(10))
        .await?;
    println!("Fetched {} contacts", contacts.contacts.len());
    for contact in contacts.contacts.iter() {
        let key = contact
            .public_key
            .iter()
            .take(6)
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let hops = if contact.out_path_len < 0 {
            "Flood".to_string()
        } else {
            format!("{} hop", contact.out_path_len)
        };
        println!(
            "{}  {:5}  {:6}  {}",
            key,
            contact.contact_type.to_string(),
            hops,
            contact.adv_name
        );
    }

    Ok(())
}

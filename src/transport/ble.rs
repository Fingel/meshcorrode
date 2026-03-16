use btleplug::api::{
    Central, CentralEvent, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use bytes::Bytes;
use futures::StreamExt;
use log::debug;
use tokio::sync::mpsc;
use uuid::{Uuid, uuid};

use super::Transport;

const UART_SERVICE_UUID: Uuid = uuid!("6E400001-B5A3-F393-E0A9-E50E24DCCA9E");
const UART_RX_CHAR_UUID: Uuid = uuid!("6E400002-B5A3-F393-E0A9-E50E24DCCA9E");
const UART_TX_CHAR_UUID: Uuid = uuid!("6E400003-B5A3-F393-E0A9-E50E24DCCA9E");

#[derive(thiserror::Error, Debug)]
pub enum BleError {
    #[error("btleplug: {0}")]
    Btleplug(#[from] btleplug::Error),
    #[error("no bluetooth adapter found")]
    NoAdapter,
    #[error("no matching MeshCore device found")]
    DeviceNotFound,
    #[error("UART characteristic not found on device")]
    CharacteristicNotFound,
    #[error("not connected")]
    NotConnected,
}

// How to identify the companion device during a scan.
pub enum BleFilter {
    /// Accept any device with a name starting with "MeshCore"
    /// This is the default for MesChore companion firmware
    AnyMeshCore,
    /// Accept any device containing the provided name.
    NameContains(String),
    /// Connect to an exact BT address e.g. "DE:AD:BE:EF:CA:FE".
    Address(String),
}

pub struct BleTransport {
    filter: BleFilter,
    peripheral: Option<Peripheral>,
    rx_char: Option<Characteristic>,
}

impl BleTransport {
    pub fn new(filter: BleFilter) -> Self {
        Self {
            filter,
            peripheral: None,
            rx_char: None,
        }
    }

    /// Determine if a peripheral matches the filter criteria.
    fn matches(&self, name: Option<&str>, addr: &str) -> bool {
        match &self.filter {
            BleFilter::AnyMeshCore => name.is_some_and(|n| n.starts_with("MeshCore")),
            BleFilter::NameContains(s) => name.is_some_and(|n| n.contains(s)),
            BleFilter::Address(a) => addr.eq_ignore_ascii_case(a),
        }
    }
}

impl Transport for BleTransport {
    type Error = BleError;

    async fn connect(&mut self) -> Result<mpsc::Receiver<Bytes>, BleError> {
        // Code apadted from btleplug repo example: examples/event_driven_discovery.rs
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        // We could support multiple adapters here, for now just pick the first one.
        debug!("found {} adapters", adapters.len());
        let central: Adapter = adapters.into_iter().nth(0).ok_or(BleError::NoAdapter)?;

        central
            .start_scan(ScanFilter {
                services: vec![UART_SERVICE_UUID],
            })
            .await?;

        let peripheral = {
            let mut events = central.events().await?;
            let mut found: Option<Peripheral> = None;

            while let Some(event) = events.next().await {
                if let CentralEvent::DeviceDiscovered(id) = event {
                    let p = central.peripheral(&id).await?;
                    let props = p.properties().await?.unwrap_or_default();
                    let name = props.local_name.as_deref();
                    let addr = props.address.to_string();
                    debug!("disc. device: id:{} name: {:?} addr: {}", id, name, addr);
                    if self.matches(name, &addr) {
                        found = Some(p);
                        break;
                    }
                }
            }

            central.stop_scan().await?;
            found.ok_or(BleError::DeviceNotFound)?
        };
        peripheral.connect().await?;
        // See btleplug/examples/subscribe_notify.rs#65
        peripheral.discover_services().await?;
        let chars = peripheral.characteristics();
        let tx_char = chars
            .iter()
            .find(|c| c.uuid == UART_TX_CHAR_UUID)
            .ok_or(BleError::CharacteristicNotFound)?;

        let rx_char = chars
            .iter()
            .find(|c| c.uuid == UART_RX_CHAR_UUID)
            .cloned()
            .ok_or(BleError::CharacteristicNotFound)?;

        peripheral.subscribe(tx_char).await?;
        let (tx, rx) = mpsc::channel::<Bytes>(8);
        let mut notifications = peripheral.notifications().await?;

        // This is the background task that actually sends data
        tokio::spawn(async move {
            // This isn't super clear but the tx.send in the if statement does the send, we check if it returns
            // an error and the break is the error condition which ends the task and closes the channel.
            while let Some(notif) = notifications.next().await {
                if notif.uuid == UART_TX_CHAR_UUID
                    && tx.send(Bytes::from(notif.value)).await.is_err()
                {
                    debug!("receiver dropped, ending notification task");
                    break;
                }
            }
        });

        self.rx_char = Some(rx_char);
        self.peripheral = Some(peripheral);
        Ok(rx)
    }

    async fn send(&self, data: &[u8]) -> Result<(), BleError> {
        let peripheral = self.peripheral.as_ref().ok_or(BleError::NotConnected)?;
        let rx_char = self.rx_char.as_ref().ok_or(BleError::NotConnected)?;
        debug!("-> {} bytes", data.len());
        peripheral
            .write(rx_char, data, WriteType::WithResponse)
            .await?;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), BleError> {
        if let Some(p) = self.peripheral.take() {
            let _ = p.disconnect().await;
        }
        self.rx_char = None;
        Ok(())
    }
}

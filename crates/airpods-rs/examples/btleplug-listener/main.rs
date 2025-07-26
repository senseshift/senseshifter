use airpods_rs::continuity::{ContinuityMessage, CONTINUITY_MANUFACTURER_ID};

use btleplug::api::{
    bleuuid::BleUuid, Central, CentralEvent, Manager as _, Peripheral, ScanFilter,
};
use btleplug::platform::{Adapter, Manager, PeripheralId};
use bytes::Bytes;
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = Manager::new().await?;
    let central = manager.adapters().await?.into_iter().next().unwrap();

    let mut events = central.events().await?;

    central.start_scan(ScanFilter::default()).await?;

    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(id) | CentralEvent::DeviceUpdated(id) => {
                // if let Err(e) = handle_peripheral_event(&id, &central).await {
                //     eprintln!("Error handling peripheral event: {}", e);
                // }
                match handle_peripheral_event(&id, &central).await {
                    Ok(Some(message)) => {
                        println!("Discovered or updated peripheral: {:?}, Message: {:?}", id, message);
                    },
                    Err(e) => {
                        eprintln!("Error handling peripheral event: {}", e);
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(())
}

async fn handle_peripheral_event(
    peripheral_id: &PeripheralId,
    adapter: &Adapter,
) -> anyhow::Result<Option<ContinuityMessage>> {
    let peripheral = adapter.peripheral(peripheral_id).await?;
    let properties = match peripheral.properties().await? {
        Some(props) => props,
        None => return Ok(None),
    };

    let message = match properties.manufacturer_data.get(&CONTINUITY_MANUFACTURER_ID) {
        Some(data) => data.clone(),
        None => return Ok(None),
    };
    let message = match ContinuityMessage::try_from(Bytes::from(message.clone())) {
        Ok(msg) => msg,
        Err(e) => return Ok(None),
    };

    Ok(Some(message))
}
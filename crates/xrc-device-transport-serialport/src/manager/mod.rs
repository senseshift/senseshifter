use async_trait::async_trait;
use std::time::Duration;
use tokio::sync::mpsc;
use xrc_device_manager::api::*;
use xrc_device_manager::Result;

#[derive(Default)]
pub struct SerialPortTransportManagerBuilder {}

impl TransportManagerBuilder for SerialPortTransportManagerBuilder {
  fn finish(
    &self,
    event_sender: mpsc::Sender<TransportManagerEvent>,
  ) -> xrc_device_manager::Result<Box<dyn TransportManager>> {
    let manager = SerialPortTransportManager { event_sender };
    Ok(Box::new(PeriodicScanTransportManager::new(manager)))
  }
}

pub struct SerialPortTransportManager {
  event_sender: mpsc::Sender<TransportManagerEvent>,
}

#[async_trait]
impl RescanTransportManager for SerialPortTransportManager {
  fn name(&self) -> &'static str {
    "serialport"
  }

  fn rescan_wait_duration(&self) -> Duration {
    Duration::from_secs(5)
  }

  fn ready(&self) -> bool {
    true
  }

  async fn scan(&self) -> Result<()> {
    let ports = tokio_serial::available_ports()?;

    // info!("Found {} serial ports: {:?}", ports.len(), ports);

    Ok(())
  }

  fn devices(&self) -> Result<Vec<ConcurrentDevice>> {
    todo!()
  }

  fn get_device(&self, device_id: &DeviceId) -> Result<Option<ConcurrentDevice>> {
    todo!()
  }
}

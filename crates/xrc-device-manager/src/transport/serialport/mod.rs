use std::time::Duration;
use anyhow::Result;
use log::info;
use tokio::sync::mpsc::Sender;
use tracing::error;
use crate::transport::api::{PeriodicScanTransportManager, RawSerialDevice, RescanTransportManager, TransportManager, TransportManagerBuilder, TransportManagerEvent};

pub(crate) struct SerialPortManagerBuilder {

}

impl Default for SerialPortManagerBuilder {
  fn default() -> Self {
    Self {}
  }
}

impl TransportManagerBuilder for SerialPortManagerBuilder {
  fn finish(&self, event_sender: Sender<TransportManagerEvent>) -> Result<Box<dyn TransportManager>> {
    Ok(Box::new(PeriodicScanTransportManager::new(SerialPortManager {})))
  }
}

pub(crate) struct SerialPortManager {

}

#[async_trait::async_trait]
impl RescanTransportManager for SerialPortManager {
  fn name(&self) -> &'static str {
    "serialport"
  }

  fn ready(&self) -> bool {
    true
  }

  fn rescan_wait_duration(&self) -> Duration {
    Duration::from_secs(5)
  }

  async fn scan(&self) -> Result<()> {
    error!("Serial port manager does not support scanning.");
    Ok(())
  }

  async fn raw_serial_devices(&self) -> Result<Option<Vec<RawSerialDevice>>> {
    Ok(Some(
      serialport::available_ports()?
        .into_iter()
        .map(|port| {
          RawSerialDevice::Serial {
            port: port.port_name,
          }
        })
        .collect()
    ))
  }
}
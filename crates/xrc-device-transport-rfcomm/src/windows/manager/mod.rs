use async_trait::async_trait;
use std::time::Duration;
use tokio::sync::mpsc;
use xrc_device_manager::api::*;
use xrc_device_manager::Result;

use tracing::{info, instrument, warn};
use windows::Devices::Bluetooth::Rfcomm::RfcommServiceId;
use windows::Devices::Bluetooth::{BluetoothAdapter, BluetoothDevice, Rfcomm::RfcommDeviceService};
use windows::Devices::Enumeration::DeviceInformation;

#[derive(Default)]
pub struct WindowsRfcommTransportManagerBuilder {}

impl TransportManagerBuilder for WindowsRfcommTransportManagerBuilder {
  fn finish(
    &self,
    _event_sender: mpsc::Sender<TransportManagerEvent>,
  ) -> Result<Box<dyn TransportManager>> {
    let inner = WindowsRfcommTransportManager {};
    Ok(Box::new(PeriodicScanTransportManager::new(inner)))
  }
}

pub struct WindowsRfcommTransportManager {}

#[async_trait]
impl RescanTransportManager for WindowsRfcommTransportManager {
  fn name(&self) -> &'static str {
    "rfcomm"
  }

  fn rescan_wait_duration(&self) -> Duration {
    Duration::from_secs(10)
  }

  fn ready(&self) -> bool {
    todo!()
  }

  #[instrument(skip(self))]
  async fn scan(&self) -> Result<()> {
    let adapter = BluetoothAdapter::GetDefaultAsync()?.await?;
    if !adapter.IsClassicSupported()? {
      warn!("Bluetooth classic is not supported on this device");
      return Ok(());
    }

    let service_id = RfcommServiceId::SerialPort()?;
    let selector = RfcommDeviceService::GetDeviceSelector(&service_id)?;

    // info!("Selector: {}", selector);

    let devices = DeviceInformation::FindAllAsyncAqsFilter(&selector)?.await?;

    let id = devices.GetAt(0)?.Id()?;
    let device = BluetoothDevice::FromIdAsync(&id)?.await?;

    info!("Found device: {:?}", device.Name()?);

    let rfcomm_services = device.GetRfcommServicesAsync()?.await?;
    for service in rfcomm_services.Services()? {
      info!("Found Rfcomm Service: {:?}", service.ServiceId()?);
    }

    Ok(())
  }

  async fn connect_scanned(&self, _device_id: &DeviceId) -> Result<()> {
    todo!()
  }

  fn devices(&self) -> Result<Vec<ConcurrentDevice>> {
    todo!()
  }

  fn get_device(&self, device_id: &DeviceId) -> Result<Option<ConcurrentDevice>> {
    todo!()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_scan() {
    tracing_subscriber::fmt::init();

    let manager = WindowsRfcommTransportManager {};
    manager.scan().await.unwrap();
  }
}

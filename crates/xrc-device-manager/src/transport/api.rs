use std::sync::Arc;
use anyhow::Result;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use tracing::error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawSerialDevice {
  Serial {
    port: String,
  },
  RFComm {
    address: String,
    name: String,
  },
  BluetoothLE {
    address: String,
    name: String,
  },
}

impl Into<xrconnect_proto::devices::v1alpha1::RawDevice> for RawSerialDevice {
  fn into(self) -> xrconnect_proto::devices::v1alpha1::RawDevice {
    match self {
      RawSerialDevice::Serial { port } => {
        xrconnect_proto::devices::v1alpha1::RawDevice {
          r#type: Some(xrconnect_proto::devices::v1alpha1::raw_device::Type::Serial(xrconnect_proto::devices::v1alpha1::raw_device::Serial{
            port,
          }))
        }
      }
      RawSerialDevice::BluetoothLE { address, name } => {
        xrconnect_proto::devices::v1alpha1::RawDevice {
          r#type: Some(xrconnect_proto::devices::v1alpha1::raw_device::Type::BluetoothLE(xrconnect_proto::devices::v1alpha1::raw_device::BluetoothLE{
            address,
            name,
          }))
        }
      }
      RawSerialDevice::RFComm { address, name } => {
        xrconnect_proto::devices::v1alpha1::RawDevice {
          r#type: Some(xrconnect_proto::devices::v1alpha1::raw_device::Type::RFComm(xrconnect_proto::devices::v1alpha1::raw_device::RFComm{
            address,
            name,
          }))
        }
      }
    }
  }
}

#[derive(Debug)]
pub enum TransportManagerEvent {
  ScanFinished,
  DeviceDiscovered,
}

pub trait TransportManagerBuilder: Send {
  fn finish(&self, event_sender: mpsc::Sender<TransportManagerEvent>) -> Result<Box<dyn TransportManager>>;
}

#[async_trait::async_trait]
pub trait TransportManager: Send + Sync {
  fn name(&self) -> &'static str;

  async fn scan_start(&mut self) -> Result<()>;

  async fn scan_stop(&mut self) -> Result<()>;

  fn is_scanning(&self) -> bool;

  fn ready(&self) -> bool;

  async fn raw_serial_devices(&self) -> Result<Option<Vec<RawSerialDevice>>> {
    Ok(None)
  }
}

#[async_trait::async_trait]
pub trait RescanTransportManager: Sync + Send {
  fn name(&self) -> &'static str;

  fn ready(&self) -> bool;

  fn rescan_wait_duration(&self) -> std::time::Duration;

  async fn scan(&self) -> Result<()>;

  async fn raw_serial_devices(&self) -> Result<Option<Vec<RawSerialDevice>>> {
    Ok(None)
  }
}

pub(crate) struct PeriodicScanTransportManager<T: RescanTransportManager + 'static> {
  inner: Arc<T>,
  cancel_token: Option<CancellationToken>,
}

impl<T: RescanTransportManager> PeriodicScanTransportManager<T> {
  pub fn new(inner: T) -> Self {
    Self {
      inner: Arc::new(inner),
      cancel_token: None,
    }
  }
}

#[async_trait::async_trait]
impl<T: RescanTransportManager> TransportManager for PeriodicScanTransportManager<T> {
  fn name(&self) -> &'static str {
    self.inner.name()
  }

  async fn scan_start(&mut self) -> Result<()> {
    if self.cancel_token.is_some() {
      return Ok(())
    }

    let cancel_token = CancellationToken::new();
    let child_token = cancel_token.child_token();
    self.cancel_token = Some(cancel_token);

    let inner = self.inner.clone();
    tokio::spawn(async move {
      loop {
        if let Err(err) = inner.scan().await {
          error!("PeriodicScanTransportManager Failure: {}", err);
          break;
        }

        // Wait for the next scan or cancellation.
        tokio::select! {
          _ = tokio::time::sleep(inner.rescan_wait_duration()) => continue,
          _ = child_token.cancelled() => break,
        }
      }
    });

    Ok(())
  }

  async fn scan_stop(&mut self) -> Result<()> {
    if self.cancel_token.is_none() {
      return Ok(())
    }
    Ok(self.cancel_token.take().unwrap().cancel())
  }

  fn is_scanning(&self) -> bool {
    self.cancel_token.is_some()
  }

  fn ready(&self) -> bool {
    self.inner.ready()
  }

  async fn raw_serial_devices(&self) -> Result<Option<Vec<RawSerialDevice>>> {
    self.inner.raw_serial_devices().await
  }
}
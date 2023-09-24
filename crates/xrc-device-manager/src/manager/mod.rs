use std::{
  pin::Pin,
  result::Result as StdResult,
};
use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;
use anyhow::Result;
use tracing::{debug, error};

#[cfg(feature = "tonic")]
use tonic::{Request, Response, Status, async_trait};
#[cfg(all(feature = "tonic", feature = "proto-v1alpha1"))]
use xrconnect_proto::devices::v1alpha1::device_manager_server::{
  DeviceManager as DeviceManagerRPC,
};
#[cfg(feature = "proto-v1alpha1")]
use xrconnect_proto::{
  devices::{
    v1alpha1::{
      DeviceMessage,
      device_message,
      // Event Streaming
      EventStreamRequest,
      EventStreamResponse,
      // Scanning
      ScanStartRequest,
      ScanStartResponse,
      ScanStopRequest,
      ScanStopResponse,
      // Device Actions
      DeviceAddRequest,
      DeviceAddResponse,
      DeviceConnectRequest,
      DeviceDisconnectRequest,
      DeviceConnectResponse,
      DeviceDisconnectResponse,
    },
  }
};

use async_stream::stream;
use futures::{pin_mut, Stream};
use futures_util::future::join_all;
use tracing::info;
use tokio::sync::{broadcast, mpsc, oneshot, RwLock, RwLockReadGuard, TryLockError};
use tokio_util::sync::CancellationToken;

mod task;
use task::DeviceManagerTask;

use xrconnect_proto::devices::v1alpha1::{RawDeviceRequest, RawDeviceResponse};
use xrc_transport::api::{Device, TransportManager, TransportManagerBuilder};

pub(super) enum DeviceManagerCommand {
  ScanStart(oneshot::Sender<Result<()>>),
  ScanStop(oneshot::Sender<Result<()>>),
}

impl std::fmt::Debug for DeviceManagerCommand {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DeviceManagerCommand::ScanStart(_) => write!(f, "ScanStart"),
      DeviceManagerCommand::ScanStop(_) => write!(f, "ScanStop"),
    }
  }
}

#[derive(Debug, Clone)]
pub enum DeviceManagerEvent {
  ScanStarted,
  ScanStopped,
  DeviceDiscovered {
    device_id: String,
    device: Arc<RwLock<Box<dyn Device>>>,
  },
  DeviceUpdated {
    device_id: String,
    device: Arc<RwLock<Box<dyn Device>>>,
  },
}

impl PartialEq for DeviceManagerEvent {
  fn eq(&self, other: &Self) -> bool {
    match self {
      DeviceManagerEvent::ScanStarted => {
        matches!(other, DeviceManagerEvent::ScanStarted)
      }
      DeviceManagerEvent::ScanStopped => {
        matches!(other, DeviceManagerEvent::ScanStopped)
      }
      DeviceManagerEvent::DeviceDiscovered { device_id, .. } => {
        matches!(other, DeviceManagerEvent::DeviceDiscovered { device_id: other_device_id, .. } if device_id == other_device_id)
      }
      DeviceManagerEvent::DeviceUpdated { device_id, .. } => {
        matches!(other, DeviceManagerEvent::DeviceUpdated { device_id: other_device_id, .. } if device_id == other_device_id)
      }
    }
  }
}

#[cfg(feature = "proto-v1alpha1")]
impl Into<DeviceMessage> for DeviceManagerEvent {
  fn into(self) -> DeviceMessage {
    match self {
      DeviceManagerEvent::ScanStarted => DeviceMessage {
        r#type: Some(device_message::Type::ScanStarted(device_message::ScanStarted {})),
      },
      DeviceManagerEvent::ScanStopped => DeviceMessage {
        r#type: Some(device_message::Type::ScanStopped(device_message::ScanStopped {})),
      },
      DeviceManagerEvent::DeviceDiscovered { device_id, device } => DeviceMessage {
        r#type: Some(device_message::Type::DeviceDiscovered(device_message::DeviceDiscovered {
          device_id,
          device: match device.try_read() {
            Ok(device) => Some(device.deref().deref().into()),
            Err(_) => None,
          },
        })),
      },
      DeviceManagerEvent::DeviceUpdated { device_id, device } => DeviceMessage {
        r#type: Some(device_message::Type::DeviceUpdated(device_message::DeviceUpdated {
          device_id,
          device: match device.try_read() {
            Ok(device) => Some(device.deref().deref().into()),
            Err(_) => None,
          },
        })),
      },
    }
  }
}

pub struct DeviceManagerBuilder {
  transport_managers: Vec<Box<dyn TransportManagerBuilder>>,
}

impl Default for DeviceManagerBuilder {
  fn default() -> Self {
    let mut builder = Self {
      transport_managers: Vec::new(),
    };

    #[cfg(feature = "transport-btle")]
    {
      use xrc_transport_btle::BtleManagerBuilder;
      let mut btle_builder = BtleManagerBuilder::default();

      #[cfg(feature = "devices-apple-continuity")]
      {
        use xrc_devices_apple_continuity::AppleContinuityProtocolSpecifierBuilder;
        btle_builder.specifier(AppleContinuityProtocolSpecifierBuilder::default());
      }

      #[cfg(feature = "devices-bhaptics")]
      {
        // todo: add bHaptics
      }

      #[cfg(feature = "devices-opengloves")]
      {
        // todo: add OpenGloves
      }

      builder.transport(btle_builder);
    }

    #[cfg(feature = "transport-serialport")]
    {
      // todo: add serialport

      #[cfg(feature = "devices-opengloves")]
      {
        // todo: add OpenGloves
      }
    }

    #[cfg(feature = "transport-rfcomm")]
    {
      // todo: add serialport

      #[cfg(feature = "devices-opengloves")]
      {
        // todo: add OpenGloves
      }

      #[cfg(feature = "devices-protubevr")]
      {
        // todo: add ProTubeVR
      }
    }

    builder
  }
}

impl DeviceManagerBuilder {
  pub fn transport<T: TransportManagerBuilder + 'static>(&mut self, builder: T) -> &mut Self
  {
    self.transport_managers.push(Box::new(builder));
    self
  }

  pub fn build(&self) -> Result<DeviceManager> {
    let (task_command_sender, task_command_receiver) = mpsc::channel(256);
    let (event_sender, _event_receiver) = broadcast::channel(256);
    let cancel_token = CancellationToken::new();

    let (transport_event_sender, transport_event_receiver) = mpsc::channel(256);
    let transport_managers: Vec<_> = self.transport_managers
      .iter()
      .map(|builder| -> Result<Box<dyn TransportManager>> {
        builder.finish(transport_event_sender.clone())
      })
      .collect::<Result<Vec<_>, _>>()?;

    let task = DeviceManagerTask {
      transport_managers,
      command_receiver: task_command_receiver,
      transport_event_receiver,
      event_sender: event_sender.clone(),
      cancel_token: cancel_token.child_token(),
    };
    tokio::spawn(async move {
      pin_mut!(task);
      if let Err(err) = task.run().await {
        error!("Device manager task exited with error: {}", err);
      }
    });

    let manager = DeviceManager {
      task_command_sender,
      event_sender,
      cancel_token,
    };

    Ok(manager)
  }
}

pub struct DeviceManager {
  task_command_sender: mpsc::Sender<DeviceManagerCommand>,
  event_sender: broadcast::Sender<DeviceManagerEvent>,
  cancel_token: CancellationToken,
}

impl Default for DeviceManager {
  fn default() -> Self {
    let mut builder = Self::builder();

    builder
      .build()
      .expect("Default is infallible")
  }
}

impl DeviceManager {
  pub fn builder() -> DeviceManagerBuilder {
    DeviceManagerBuilder::default()
  }

  pub fn event_stream(&self) -> impl Stream<Item=DeviceManagerEvent> {
    let receiver = self.event_sender.subscribe();
    stream! {
      pin_mut!(receiver);
      while let Ok(event) = receiver.recv().await {
        yield event;
      }
    }
  }

  pub async fn scan_start(&self) -> Result<()> {
    let (sender, receiver) = oneshot::channel();

    self.task_command_sender.send(DeviceManagerCommand::ScanStart(sender)).await?;
    receiver.await?
  }

  pub async fn scan_stop(&self) -> Result<()> {
    let (sender, receiver) = oneshot::channel();

    self.task_command_sender.send(DeviceManagerCommand::ScanStop(sender)).await?;
    receiver.await?
  }
}

impl Drop for DeviceManager {
  fn drop(&mut self) {
    self.cancel_token.cancel();
  }
}

#[cfg(feature = "tonic")]
#[async_trait]
impl DeviceManagerRPC for DeviceManager {
  type EventStreamStream = Pin<Box<dyn Stream<Item=StdResult<EventStreamResponse, Status>> + Send>>;

  async fn event_stream(&self, _request: Request<EventStreamRequest>) -> StdResult<Response<Self::EventStreamStream>, Status> {
    let receiver = self.event_sender.subscribe();

    let stream = stream! {
      pin_mut!(receiver);
      while let Ok(event) = receiver.recv().await {
        yield Ok(EventStreamResponse {
          message: Some(event.into()),
        });
      }
    };

    Ok(Response::new(Box::pin(stream) as Self::EventStreamStream))
  }

  async fn scan_start(&self, _request: Request<ScanStartRequest>) -> StdResult<Response<ScanStartResponse>, Status> {
    self.scan_start().await
      .map(|_| Response::new(ScanStartResponse {}))
      .map_err(|err| Status::from_error(Box::from(err)))
  }

  async fn scan_stop(&self, _request: Request<ScanStopRequest>) -> StdResult<Response<ScanStopResponse>, Status> {
    self.scan_stop().await
      .map(|_| Response::new(ScanStopResponse {}))
      .map_err(|err| Status::from_error(Box::from(err)))
  }

  async fn device_connect(&self, request: Request<DeviceConnectRequest>) -> StdResult<Response<DeviceConnectResponse>, Status> {
    todo!()
  }

  async fn device_disconnect(&self, request: Request<DeviceDisconnectRequest>) -> StdResult<Response<DeviceDisconnectResponse>, Status> {
    todo!()
  }

  async fn device_add(&self, request: Request<DeviceAddRequest>) -> StdResult<Response<DeviceAddResponse>, Status> {
    todo!()
  }

  async fn get_raw_devices(&self, request: Request<RawDeviceRequest>) -> StdResult<Response<RawDeviceResponse>, Status> {
    todo!()
  }
}

#[cfg(test)]
mod tests {
  use futures_util::StreamExt;
  use super::*;

  #[tokio::test]
  async fn test_event_stream() {
    let (event_sender, _event_receiver) = broadcast::channel(1);

    let device_manager = DeviceManager {
      task_command_sender: mpsc::channel(1).0,
      event_sender: event_sender.clone(),
      cancel_token: CancellationToken::new(),
    };

    let event_stream = Box::pin(device_manager.event_stream());

    let _ = event_sender.send(DeviceManagerEvent::ScanStarted);

    let (event, _) = event_stream.into_future().await;
    assert_eq!(event.unwrap(), DeviceManagerEvent::ScanStarted);
  }
}
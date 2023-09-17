use std::{
  pin::Pin,
  result::Result as StdResult,
};
use anyhow::Result;
use tracing::{error};

use xrconnect_proto::{
  devices::v1alpha1::{
    DeviceMessage,
  },
};

#[cfg(feature = "tonic")]
use xrconnect_proto::{
  devices::v1alpha1::{
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

    device_manager_server::{
      DeviceManager as DeviceManagerRPC,
    },
  },
};
#[cfg(feature = "tonic")]
use tonic::{Request, Response, Status, async_trait};

use async_stream::stream;
use futures::{pin_mut, Stream};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_util::sync::CancellationToken;

mod task;
use task::DeviceManagerTask;

#[derive(Debug)]
pub(super) enum DeviceManagerCommand {
  ScanStart(oneshot::Sender<Result<()>>),
  ScanStop(oneshot::Sender<Result<()>>),
}

pub struct DeviceManagerBuilder {}

impl Default for DeviceManagerBuilder {
  fn default() -> Self {
    Self {}
  }
}

impl DeviceManagerBuilder {
  pub fn build(&self) -> Result<DeviceManager> {
    let (task_command_sender, task_command_receiver) = mpsc::unbounded_channel();
    let (event_sender, _event_receiver) = broadcast::channel(256);
    let cancel_token = CancellationToken::new();

    let mut task = DeviceManagerTask {
      command_receiver: task_command_receiver,
      event_sender: event_sender.clone(),
      cancel_token: cancel_token.child_token(),
    };
    tokio::spawn(async move {
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
  task_command_sender: mpsc::UnboundedSender<DeviceManagerCommand>,
  event_sender: broadcast::Sender<DeviceMessage>,
  cancel_token: CancellationToken,
}

impl Default for DeviceManager {
  fn default() -> Self {
    Self::builder().build().expect("Default is infallible")
  }
}

impl DeviceManager {
  pub fn builder() -> DeviceManagerBuilder {
    DeviceManagerBuilder::default()
  }

  pub fn event_stream(&self) -> impl Stream<Item=DeviceMessage> {
    let receiver = self.event_sender.subscribe();
    stream! {
      pin_mut!(receiver);
      while let Ok(event) = receiver.recv().await {
        yield event;
      }
    }
  }

  pub async fn scan_start(&self) -> Result<()> {
    let command_sender = self.task_command_sender.clone();
    let (sender, receiver) = oneshot::channel();

    let _ = command_sender.send(DeviceManagerCommand::ScanStart(sender));
    receiver.await?
  }

  pub async fn scan_stop(&self) -> Result<()> {
    let command_sender = self.task_command_sender.clone();
    let (sender, receiver) = oneshot::channel();

    let _ = command_sender.send(DeviceManagerCommand::ScanStop(sender));
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
        yield Ok(EventStreamResponse::from(event));
      }
    };

    Ok(Response::new(Box::pin(stream) as Self::EventStreamStream))
  }

  async fn scan_start(&self, _request: Request<ScanStartRequest>) -> StdResult<Response<ScanStartResponse>, Status> {
    self.scan_start().await
      .map(|_| Response::new(ScanStartResponse {}))
      .map_err(|err| Status::internal(err.to_string()))
  }

  async fn scan_stop(&self, _request: Request<ScanStopRequest>) -> StdResult<Response<ScanStopResponse>, Status> {
    self.scan_stop().await
      .map(|_| Response::new(ScanStopResponse {}))
      .map_err(|err| Status::internal(err.to_string()))
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
}

#[cfg(test)]
mod tests {
  use futures_util::StreamExt;
  use xrconnect_proto::devices::v1alpha1::device_message::*;
  use super::*;

  #[tokio::test]
  async fn test_event_stream() {
    let (event_sender, _event_receiver) = broadcast::channel(1);

    let device_manager = DeviceManager {
      task_command_sender: mpsc::unbounded_channel().0,
      event_sender: event_sender.clone(),
      cancel_token: CancellationToken::new(),
    };

    let event_stream = Box::pin(device_manager.event_stream());

    let _ = event_sender.send(DeviceMessage {
      r#type: Some(Type::ScanStarted(ScanStarted {})),
    });

    let (event, _) = event_stream.into_future().await;
    assert_eq!(event.unwrap().r#type.unwrap(), Type::ScanStarted(ScanStarted {}));
  }
}
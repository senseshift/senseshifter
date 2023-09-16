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
  async_trait,
};

#[cfg(feature = "tonic")]
use xrconnect_proto::{
  devices::v1alpha1::{
    DeviceAddRequest, DeviceConnectRequest, DeviceDisconnectRequest,
    device_manager_server::{
      DeviceManager as DeviceManagerRPC,
    },
  },
};
#[cfg(feature = "tonic")]
use tonic::{Request, Response, Status};

use async_stream::stream;
use futures::{pin_mut, Stream};
use futures_util::StreamExt;
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;

mod task;
use task::DeviceManagerTask;
use xrconnect_proto::devices::v1alpha1::{DeviceAddResponse, DeviceConnectResponse, DeviceDisconnectResponse, EventStreamRequest, EventStreamResponse, ScanStartRequest, ScanStartResponse, ScanStopRequest, ScanStopResponse};

pub struct DeviceManagerBuilder {

}

impl DeviceManagerBuilder {
  pub fn build(&self) -> Result<DeviceManager> {
    let (event_sender, _event_receiver) = broadcast::channel(256);
    let cancel_token = CancellationToken::new();

    let mut task = DeviceManagerTask {
      event_sender: event_sender.clone(),
      cancel_token: cancel_token.child_token(),
    };
    tokio::spawn(async move {
      if let Err(err) = task.run().await {
        error!("Device manager task exited with error: {}", err);
      }
    });

    let manager = DeviceManager {
      event_sender,
      cancel_token,
    };

    Ok(manager)
  }
}

pub struct DeviceManager {
  event_sender: broadcast::Sender<DeviceMessage>,
  cancel_token: CancellationToken,
}

impl DeviceManager {
  pub fn event_stream(&self) -> impl Stream<Item = DeviceMessage> {
    let receiver = self.event_sender.subscribe();
    stream! {
      pin_mut!(receiver);
      while let Ok(event) = receiver.recv().await {
        yield event;
      }
    }
  }
}

#[cfg(feature = "tonic")]
#[async_trait]
impl DeviceManagerRPC for DeviceManager {
  type EventStreamStream = Pin<Box<dyn Stream<Item = StdResult<EventStreamResponse, Status>> + Send>>;

  async fn event_stream(&self, _request: Request<EventStreamRequest>) -> StdResult<Response<Self::EventStreamStream>, Status> {
    let event_stream = self.event_stream();

    let stream = stream! {
      pin_mut!(event_stream);
      while let Some(event) = event_stream.next().await {
        yield Ok(EventStreamResponse::from(event));
      }
    };

    Ok(Response::new(Box::pin(stream) as Self::EventStreamStream))
  }

  async fn scan_start(&self, request: Request<ScanStartRequest>) -> StdResult<Response<ScanStartResponse>, Status> {
    todo!()
  }

  async fn scan_stop(&self, request: Request<ScanStopRequest>) -> StdResult<Response<ScanStopResponse>, Status> {
    todo!()
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
    let (event_sender, _event_receiver) = broadcast::channel(256);

    let device_manager = DeviceManager {
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
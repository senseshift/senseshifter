use crate::api::*;
use crate::Result;
use anyhow::anyhow;
use async_stream::stream;
use dashmap::DashMap;
use derivative::Derivative;
use futures::{pin_mut, Stream};
use futures_util::StreamExt;
use std::sync::Arc;
use task::DeviceManagerTask;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument, Instrument};

mod task;

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) enum DeviceManagerCommand {
  ScanStart(#[derivative(Debug = "ignore")] oneshot::Sender<Result<()>>),
  ScanStop(#[derivative(Debug = "ignore")] oneshot::Sender<Result<()>>),
}

#[derive(Default)]
pub struct DeviceManagerBuilder {
  transport_managers: Vec<Box<dyn TransportManagerBuilder>>,
}

impl DeviceManagerBuilder {
  pub fn transport<T: TransportManagerBuilder + 'static>(&mut self, builder: T) -> &mut Self {
    self.transport_managers.push(Box::new(builder));
    self
  }

  pub fn build(self) -> Result<DeviceManager> {
    let (task_command_sender, task_command_receiver) = mpsc::channel(256);
    let (event_sender, _event_receiver) = broadcast::channel(256);
    let cancel_token = CancellationToken::new();

    let (transport_event_sender, transport_event_receiver) = mpsc::channel(256);

    let transport_managers: Vec<_> = self
      .transport_managers
      .iter()
      .map(|builder| -> Result<Box<dyn TransportManager>> {
        builder.finish(transport_event_sender.clone())
      })
      .collect::<Result<Vec<_>>>()?;

    let discovered_devices = Arc::new(DashMap::new());
    let connected_devices = Arc::new(DashMap::new());

    let task_span = tracing::info_span!("DeviceManagerTask");
    let task = DeviceManagerTask::new(
      cancel_token.clone(),
      transport_managers,
      transport_event_receiver,
      event_sender.clone(),
      task_command_receiver,
      discovered_devices.clone(),
      connected_devices.clone(),
    );
    let _join_token = tokio::spawn(
      async move {
        pin_mut!(task);
        if let Err(err) = task.run().await {
          error!("Device Manager Task failed: {:?}", err);
        }
        info!("Device Manager Task exited.");
      }
      .instrument(task_span),
    );

    let manager = DeviceManager {
      cancel_token,
      event_sender,
      task_command_sender,
      discovered_devices,
      connected_devices,
    };

    Ok(manager)
  }
}

pub struct DeviceManager {
  cancel_token: CancellationToken,
  event_sender: broadcast::Sender<DeviceManagerEvent>,
  task_command_sender: mpsc::Sender<DeviceManagerCommand>,
  discovered_devices: Arc<DashMap<DeviceId, ConcurrentDevice>>,
  connected_devices: Arc<DashMap<DeviceId, ConcurrentDevice>>,
}

impl DeviceManager {
  pub fn event_stream(&self) -> impl Stream<Item = DeviceManagerEvent> {
    let receiver = self.event_sender.subscribe();
    stream! {
      pin_mut!(receiver);
      while let Ok(event) = receiver.recv().await {
        yield event;
      }
    }
  }

  pub async fn scan_start(&self) -> Result<()> {
    let (tx, rx) = oneshot::channel();

    self
      .task_command_sender
      .send(DeviceManagerCommand::ScanStart(tx))
      .await?;

    rx.await?
  }

  pub async fn scan_stop(&self) -> Result<()> {
    let (tx, rx) = oneshot::channel();

    self
      .task_command_sender
      .send(DeviceManagerCommand::ScanStop(tx))
      .await?;

    rx.await?
  }

  #[instrument(skip(self))]
  pub async fn connect_device(&self, device_id: &DeviceId) -> Result<()> {
    info!("Connecting device: {:?}", device_id);

    let device = match self.discovered_devices.get(device_id) {
      Some(device) => device,
      None => return Err(anyhow!("Device not found")),
    };

    if !device.connectible() {
      return Err(anyhow!("Device is not connectible"));
    }

    if device.is_connected().await {
      return Err(anyhow!("Device already connected"));
    }

    // todo: remove from discovered devices if connection fails
    device.connect().await
  }
}

impl Drop for DeviceManager {
  fn drop(&mut self) {
    self.cancel_token.cancel();
  }
}

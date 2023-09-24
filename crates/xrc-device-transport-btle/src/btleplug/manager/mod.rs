use std::{
  sync::{
    Arc,
    atomic::{
      AtomicBool, Ordering,
    },
  }
};
use xrc_transport::{
  Result,
  api::{TransportManager, TransportManagerBuilder, TransportManagerEvent},
};
use futures::pin_mut;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info};

mod task;
use task::BtlePlugManagerTask;
use crate::BtleManagerBuilder;
use crate::btleplug::BtlePlugProtocolSpecifierBuilder;

#[derive(Debug)]
pub enum BtlePlugManagerCommand {
  ScanStart(oneshot::Sender<Result<()>>),
  ScanStop(oneshot::Sender<Result<()>>),
}

pub struct BtlePlugManagerBuilder {
  specifiers: Vec<Box<dyn BtlePlugProtocolSpecifierBuilder>>,
}

impl Default for BtlePlugManagerBuilder {
  fn default() -> Self {
    Self {
      specifiers: Vec::new(),
    }
  }
}

impl BtleManagerBuilder {
  pub fn specifier<T: BtlePlugProtocolSpecifierBuilder + 'static>(&mut self, builder: T) -> &mut Self {
    self.specifiers.push(Box::new(builder));
    self
  }
}

impl TransportManagerBuilder for BtlePlugManagerBuilder {
  fn finish(&self, event_sender: mpsc::Sender<TransportManagerEvent>) -> Result<Box<dyn TransportManager>> {
    let (command_sender, command_receiver) = mpsc::channel(256);
    let adapter_connected = Arc::new(AtomicBool::new(false));

    let protocol_specifiers: Vec<_> = self.specifiers
      .iter()
      .map(|builder| builder.finish())
      .collect();

    let task = BtlePlugManagerTask::new(
      command_receiver,
      event_sender.clone(),
      adapter_connected.clone(),
      protocol_specifiers,
    );

    tokio::spawn(async move {
      pin_mut!(task);
      if let Err(err) = task.run().await {
        error!("Device manager task exited with error: {}", err);
      }
    });

    Ok(Box::new(BtlePlugManager {
      command_sender,
      event_sender,
      is_scanning: AtomicBool::new(false),
      adapter_connected,
    }))
  }
}

pub struct BtlePlugManager {
  command_sender: mpsc::Sender<BtlePlugManagerCommand>,
  event_sender: mpsc::Sender<TransportManagerEvent>,
  is_scanning: AtomicBool,
  /// Determines if transport is ready to handle events.
  /// Shared between manager and task.
  adapter_connected: Arc<AtomicBool>,
}

#[xrc_transport::async_trait]
impl TransportManager for BtlePlugManager {
  fn name(&self) -> &'static str {
    "btleplug"
  }

  async fn scan_start(&mut self) -> Result<()> {
    self.is_scanning.store(true, Ordering::SeqCst);
    let (sender, receiver) = oneshot::channel();

    // send command to task
    let result = match self.command_sender.send(BtlePlugManagerCommand::ScanStart(sender)).await {
      Ok(()) => Ok(()),
      Err(err) => {
        error!("Error starting scan, cannot send to btleplug event loop: {}", err);
        self.is_scanning.store(false, Ordering::SeqCst);
        Err(err)
      }
    };

    // wait for response from task
    match receiver.await {
      Ok(result) => result,
      Err(err) => {
        error!("Error starting scan, cannot receive from btleplug task: {}", err);
        self.is_scanning.store(false, Ordering::SeqCst);
        Err(err.into())
      }
    }
  }

  async fn scan_stop(&mut self) -> Result<()> {
    self.is_scanning.store(false, Ordering::SeqCst);
    let (sender, receiver) = oneshot::channel();

    // send command to task
    let result = match self.command_sender.send(BtlePlugManagerCommand::ScanStop(sender)).await {
      Ok(()) => Ok(()),
      Err(err) => {
        error!("Error stopping scan, cannot send to btleplug event loop: {}", err);
        self.is_scanning.store(true, Ordering::SeqCst);
        Err(err)
      }
    };

    // wait for response from task
    match receiver.await {
      Ok(result) => result,
      Err(err) => {
        error!("Error stopping scan, cannot receive from btleplug task: {}", err);
        self.is_scanning.store(true, Ordering::SeqCst);
        Err(err.into())
      }
    }
  }

  fn is_scanning(&self) -> bool {
    self.is_scanning.load(Ordering::SeqCst)
  }

  fn ready(&self) -> bool {
    self.adapter_connected.load(Ordering::SeqCst)
  }
}
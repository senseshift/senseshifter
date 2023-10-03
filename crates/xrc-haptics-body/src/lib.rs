use tokio::sync::mpsc;

mod plane;
pub use plane::*;

pub enum ActuatorEvent {
  Vibrate(u8),
}

pub type ActuatorSender = mpsc::Sender<ActuatorEvent>;
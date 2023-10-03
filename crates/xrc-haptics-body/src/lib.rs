use tokio::sync::mpsc;

mod geometry;
pub use geometry::*;

mod plane;
pub use plane::*;

pub enum ActuatorEvent {
  Vibrate(u8),
}

pub type ActuatorSender = mpsc::Sender<ActuatorEvent>;
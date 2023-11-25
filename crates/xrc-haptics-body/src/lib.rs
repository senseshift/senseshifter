use tokio::sync::mpsc;

mod plane;
mod plane_state;

pub use plane::*;
pub use plane_state::*;

pub enum ActuatorEvent {
  Vibrate(u8),
}

pub type ActuatorSender = mpsc::Sender<ActuatorEvent>;
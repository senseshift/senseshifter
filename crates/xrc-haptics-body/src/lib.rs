use tokio::sync::mpsc;

mod geometry;
pub use geometry::*;

mod plane;
pub use plane::*;

pub enum ActuatorEvent {
  //
}

pub type ActuatorSender = mpsc::Sender<ActuatorEvent>;
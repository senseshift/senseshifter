use bh_haptic_definitions::HapticDefinitionsMessage;
use derivative::Derivative;
use getset::Getters;

#[cfg(feature = "ws")]
pub mod ws;

#[derive(Derivative, Debug, Clone, Getters)]
#[get = "pub"]
pub struct HapticEvent {
  name: String,
  event_time: u32,
}

impl HapticEvent {
  pub fn new(name: String, event_time: u32) -> Self {
    Self { name, event_time }
  }
}

#[derive(Derivative, Debug, Clone)]
pub enum HapticManagerCommand {
  ClientConnected {
    namespace: String,
  },

  RegisterHapticDefinitions {
    namespace: String,
    definitions: Box<HapticDefinitionsMessage>, // using box, since the message is quite large
  },

  PlayEvent {
    namespace: String,
    event_name: String,
    request_id: u32,

    start_millis: u64,

    // Intensity scale factor: 0.0-1.0
    intensity: f64,

    // Duration scale factor: 0.0-1.0
    duration: f64,

    offset_x: f64,
    offset_y: f64,
  },

  StopAll {
    namespace: String,
  },
}

#[derive(Derivative, Debug, Clone)]
pub enum HapticManagerEvent {
  HapticEventsUpdated {
    namespace: String,
    events: Vec<HapticEvent>,
  },
}

use bh_haptic_definitions::HapticDefinitionsMessage;
use derivative::Derivative;

#[cfg(feature = "ws")]
pub mod ws;

#[derive(Derivative, Debug, Clone)]
pub enum HapticManagerCommand {
  RegisterHapticDefinitions {
    namespace: String,
    definitions: Box<HapticDefinitionsMessage>, // using box, since the message is quite large
  },
  StopAll {
    namespace: String,
  },
}

#[derive(Derivative, Debug, Clone)]
pub enum HapticManagerEvent {}

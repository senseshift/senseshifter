use derivative::Derivative;

#[cfg(feature = "ws")]
pub mod ws;

#[derive(Derivative, Debug, Clone)]
pub enum HapticManagerCommand {}

#[derive(Derivative, Debug, Clone)]
pub enum HapticManagerEvent {}

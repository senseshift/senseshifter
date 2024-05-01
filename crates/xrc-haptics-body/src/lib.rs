pub use rshapes::Shape;

use strum::EnumDiscriminants;

pub type BodyHapticsGeometry = Shape<u8, u8>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyHapticsTarget {
  ChestFront,
  ChestBack,
}

pub type BodyVibrateIntensity = f32;

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(name(BodyHapticsEffectType))]
pub enum BodyHapticsEffect {
  Vibrate(BodyVibrateIntensity),
}

#[derive(Debug, Clone)]
pub struct BodyHapticsRequest {
  target: BodyHapticsTarget,
  geometry: BodyHapticsGeometry,
  effect: BodyHapticsEffect,
}

#[derive(Debug, Clone)]
pub struct BodyHapticsActuator {
  target: BodyHapticsTarget,
  effect: BodyHapticsEffectType,
  geometry: BodyHapticsGeometry,
}

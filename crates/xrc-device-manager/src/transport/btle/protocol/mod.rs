pub mod bhaptics;

pub trait BtlePlugProtocolHandlerBuilder: Send {
  fn finish(&self) -> Box<dyn BtlePlugProtocolHandler>;
}

pub trait BtlePlugProtocolHandler: Send + Sync {
  fn name(&self) -> &'static str;
}

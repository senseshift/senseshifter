use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

#[cfg(feature = "v1")]
pub(crate) mod v1;
#[cfg(feature = "v2")]
pub(crate) mod v2;
#[cfg(feature = "v3")]
pub(crate) mod v3;
#[cfg(feature = "v4")]
pub(crate) mod v4;
mod common;

pub(crate) enum Handler {
  #[cfg(feature = "v1")]
  V1,

  #[cfg(feature = "v2")]
  V2(v2::V2Handler),

  #[cfg(feature = "v3")]
  V3,

  #[cfg(feature = "v4")]
  V4(v4::V4Handler),

  Undefined,
}

impl Handler {
  pub(crate) async fn handle_message(&self, msg: TungsteniteMessage) -> crate::Result<()> {
    match self {
      #[cfg(feature = "v2")]
      Self::V2(handler) => handler.handle_message(msg).await,

      #[cfg(feature = "v4")]
      Self::V4(handler) => handler.handle_message(msg).await,

      _ => Err(anyhow::anyhow!("Handler not defined")),
    }
  }
}

use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

#[cfg(feature = "v2")]
pub(crate) mod v2;

pub(crate) enum Handler {
  #[cfg(feature = "v2")]
  V2(v2::V2Handler),
  Undefined,
}

impl Handler {
  pub(crate) async fn handle_message(&self, msg: TungsteniteMessage) -> crate::Result<()> {
    match self {
      #[cfg(feature = "v2")]
      Self::V2(handler) => handler.handle_message(msg).await,
      _ => Err(anyhow::anyhow!("Handler not defined")),
    }
  }
}

use bh_sdk::v2::{ClientMessage, ServerMessage};
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::{Message as TungsteniteMessage};
use tracing::{info, trace};

pub struct V2Handler {
  sender: UnboundedSender<TungsteniteMessage>,
}

impl V2Handler {
  pub fn new(sender: UnboundedSender<TungsteniteMessage>) -> Self {
    Self { sender }
  }

  #[tracing::instrument(skip(self, message))]
  pub async fn handle_message(&self, message: TungsteniteMessage) -> crate::Result<()> {
    let message = message.into_text()?;
    let message: ClientMessage = serde_json::from_slice(message.as_bytes())?;

    trace!("Received message: {:#?}", message);

    // handle effects registration first
    if let Some(register) = message.register() {
      // todo: actually register effects
      let keys = register
        .into_iter()
        .map(|msg| msg.key().clone())
        .collect::<Vec<_>>();

      let response = ServerMessage::RegisteredKeys(keys);
      let response = serde_json::to_string(&response)?;

      match self.sender.send(TungsteniteMessage::Text(response.into())) {
        Ok(_) => {}
        Err(err) => return Err(err.into()),
      }
    }

    // handle effects submission after registration
    if let Some(submit) = message.submit() {}

    Ok(())
  }
}

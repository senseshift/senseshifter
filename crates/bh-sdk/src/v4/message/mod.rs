use derivative::Derivative;
use strum::{EnumDiscriminants, EnumString, IntoDiscriminant, VariantNames};

#[derive(Derivative, EnumDiscriminants)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[strum_discriminants(name(SdkEncryptedMessageType))]
#[strum_discriminants(derive(EnumString, VariantNames))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
#[cfg_attr(feature = "serde", serde_with::serde_as)]
pub enum SdkEncryptedMessage {
  ServerKey { key: String },
  SdkClientKey { key: String },
  SdkData { data: String },
}

impl SdkEncryptedMessage {
  pub fn server_key(key: String) -> Self {
    Self::ServerKey { key }
  }

  pub fn sdk_client_key(key: String) -> Self {
    Self::SdkClientKey { key }
  }

  pub fn sdk_data(data: String) -> Self {
    Self::SdkData { data }
  }

  pub fn r#type(&self) -> SdkEncryptedMessageType {
    self.discriminant()
  }

  pub fn key(&self) -> Option<&str> {
    match self {
      Self::ServerKey { key } => Some(key),
      Self::SdkClientKey { key } => Some(key),
      Self::SdkData { .. } => None,
    }
  }

  pub fn data(&self) -> Option<&str> {
    match self {
      Self::SdkData { data } => Some(data),
      _ => None,
    }
  }
}

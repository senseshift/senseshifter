/// From observations, the `/v4/feedback` SDK uses the same messages as the `/v3/feedback`,
/// but just wraps in an outer encryption layer.
///
/// The PKI is used to only safely exchange the client's AES-GCM key.
/// All later messages are always encrypted using AES-GCM with the key above.
///
/// ## Message Flow
///
/// All the messages starting with the first SdkData are the same as in v3.
///
/// ```text
///     ┌──────┐                                                                                                               ┌───┐
//      │Server│                                                                                                               │SDK│
//      └───┬──┘                                                                                                               └─┬─┘
//          │                                  {"Type": "ServerKey", "Key": public_key_spki)}                                    │
//          │───────────────────────────────────────────────────────────────────────────────────────────────────────────────────>│
//          │                                                                                                                    │
//          │              {"Type": "SdkClientKey", "Key": encrypt(PKCS#1 v1.5, public_key_spki, client_aes_key) }               │
//          │<───────────────────────────────────────────────────────────────────────────────────────────────────────────────────│
//          │                                                                                                                    │
//          │{"Type": "SdkData", "Data": encrypt(AES-GSM, random_nonce[u8; 12], client_aes_key, "{\"type\": \"ServerReady\"}") } │
//          │───────────────────────────────────────────────────────────────────────────────────────────────────────────────────>│
//      ┌───┴──┐                                                                                                               ┌─┴─┐
//      │Server│                                                                                                               │SDK│
//      └──────┘
/// ```
use derivative::Derivative;
use strum::{EnumDiscriminants, EnumString, IntoDiscriminant, VariantNames};

#[derive(Derivative, EnumDiscriminants)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[strum_discriminants(name(SdkEncryptedMessageType))]
#[strum_discriminants(derive(EnumString, VariantNames))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "Type"))]
#[cfg_attr(feature = "serde", serde_with::serde_as)]
pub enum SdkEncryptedMessage {
  /// The server sends this message to the client to initiate the encryption handshake.
  /// The key is usually a base64-encoded RSA 2048 public key (SPKI format).
  ServerKey {
    #[cfg_attr(feature = "serde", serde(rename = "Key"))]
    key: String,
  },

  /// The client sends this message to the server to finish the encryption handshake.
  /// The key is PKCS#1 v1.5-encrypted AES-GCM key, base64-encoded.
  SdkClientKey {
    #[cfg_attr(feature = "serde", serde(rename = "Key"))]
    key: String,
  },

  /// Actual data from the SDK Client. Encrypted with the AES-GCM key above and base64-encoded.
  /// Inside, it is a JSON string with the [crate::v3::SdkMessage] messages
  SdkData {
    #[cfg_attr(feature = "serde", serde(rename = "Data"))]
    data: String,
  },
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

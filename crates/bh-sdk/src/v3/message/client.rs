use bh_haptic_definitions::{
  DevicePosition, HapticDefinitionsMessage, PathPoint, SdkApiResponseV3,
};

use derivative::Derivative;
use getset::Getters;
use strum::{EnumDiscriminants, EnumString, VariantNames};

#[derive(Derivative, EnumDiscriminants)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[strum_discriminants(name(SdkMessageType))]
#[strum_discriminants(derive(EnumString, VariantNames))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "message"))]
#[cfg_attr(feature = "serde", serde_with::serde_as)]
#[allow(clippy::large_enum_variant)] // todo: Analyze impact (#7)
pub enum SdkMessage {
  SdkRequestAuthInit(
    #[cfg_attr(feature = "serde", serde(with = "serde_handy::as_json_or_object"))]
    SdkRequestAuthInitMessage,
  ),
  SdkRequestAuth(
    #[cfg_attr(feature = "serde", serde(with = "serde_handy::as_json_or_object"))]
    SdkRequestAuthMessage,
  ),
  SdkPlayWithStartTime(
    #[cfg_attr(feature = "serde", serde(with = "serde_handy::as_json_or_object"))]
    SdkPlayWithStartTimeMessage,
  ),
  SdkPlayDotMode(
    #[cfg_attr(feature = "serde", serde(with = "serde_handy::as_json_or_object"))]
    SdkPlayDotModeMessage,
  ),
  SdkPlayPathMode(
    #[cfg_attr(feature = "serde", serde(with = "serde_handy::as_json_or_object"))]
    SdkPlayPathModeMessage,
  ),

  SdkPingAll,

  SdkStopAll,
  SdkStopByEventId(String),
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserialize<'de> for SdkMessage {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::de::Deserializer<'de>,
  {
    use serde::de::{Error, Unexpected};
    use std::str::FromStr;

    let val = serde_json::Value::deserialize(deserializer)?;
    let obj = val
      .as_object()
      .ok_or_else(|| Error::custom("SdkMessage must be a JSON object"))?;

    // accept both "type" and "Type"
    let tag = obj
      .get("type")
      .or_else(|| obj.get("Type"))
      .ok_or_else(|| Error::custom(r#"missing "type"/"Type" tag"#))?;

    let tag = tag
      .as_str()
      .ok_or_else(|| Error::invalid_type(Unexpected::Other("non-string tag"), &"a string"))?;

    let tag = SdkMessageType::from_str(tag)
      .map_err(|_| Error::unknown_variant(tag, SdkMessageType::VARIANTS))?;

    // let tag = SdkMessageType::rep

    // message may be a stringified JSON or a plain object
    let msg_v = obj
      .get("message")
      .or_else(|| obj.get("Message"))
      .cloned()
      .unwrap_or(serde_json::Value::Null);

    // helper: parse message value
    fn parse_msg<T, E>(v: serde_json::Value) -> Result<T, E>
    where
      T: serde::de::DeserializeOwned,
      E: Error,
    {
      match v {
        serde_json::Value::String(s) => serde_json::from_str::<T>(&s).map_err(E::custom),
        other => serde_json::from_value::<T>(other).map_err(E::custom),
      }
    }

    match tag {
      SdkMessageType::SdkRequestAuth => parse_msg(msg_v).map(SdkMessage::SdkRequestAuth),
      SdkMessageType::SdkRequestAuthInit => parse_msg(msg_v).map(SdkMessage::SdkRequestAuthInit),
      SdkMessageType::SdkPlayWithStartTime => {
        parse_msg(msg_v).map(SdkMessage::SdkPlayWithStartTime)
      }
      SdkMessageType::SdkPlayDotMode => parse_msg(msg_v).map(SdkMessage::SdkPlayDotMode),
      SdkMessageType::SdkPlayPathMode => parse_msg(msg_v).map(SdkMessage::SdkPlayPathMode),
      SdkMessageType::SdkPingAll => Ok(SdkMessage::SdkPingAll),
      SdkMessageType::SdkStopAll => Ok(SdkMessage::SdkStopAll),
      SdkMessageType::SdkStopByEventId => parse_msg(msg_v).map(SdkMessage::SdkStopByEventId),
    }
  }
}

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SdkRequestAuthInitMessage {
  authentication: SdkRequestAuthMessage,
  haptic: SdkApiResponseV3<HapticDefinitionsMessage>,
}

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SdkRequestAuthMessage {
  cipher: String,
  application_id: String,
  nonce_hash_value: String,
  application_id_hash_value: String,
  sdk_api_key: String,
}

impl SdkRequestAuthMessage {
  pub fn new(
    cipher: String,
    application_id: String,
    nonce_hash_value: String,
    application_id_hash_value: String,
    sdk_api_key: String,
  ) -> Self {
    Self {
      cipher,
      application_id,
      nonce_hash_value,
      application_id_hash_value,
      sdk_api_key,
    }
  }
}

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SdkPlayWithStartTimeMessage {
  event_name: String,
  request_id: u32,

  #[cfg_attr(
    feature = "serde",
    serde(default = "SdkPlayWithStartTimeMessage::default_start_millis")
  )]
  start_millis: u64,

  /// Intensity scale factor: 0.0-1.0
  #[cfg_attr(
    feature = "serde",
    serde(default = "SdkPlayWithStartTimeMessage::default_intensity")
  )]
  intensity: f64,

  /// Duration scale factor: 0.0-1.0
  #[cfg_attr(
    feature = "serde",
    serde(default = "SdkPlayWithStartTimeMessage::default_duration")
  )]
  duration: f64,

  #[cfg_attr(
    feature = "serde",
    serde(default = "SdkPlayWithStartTimeMessage::default_offset_angle_x")
  )]
  offset_angle_x: f64,

  #[cfg_attr(
    feature = "serde",
    serde(default = "SdkPlayWithStartTimeMessage::default_offset_y")
  )]
  offset_y: f64,
}

impl SdkPlayWithStartTimeMessage {
  pub fn new(
    event_name: String,
    request_id: u32,
    start_millis: u64,
    intensity: f64,
    duration: f64,
    offset_angle_x: f64,
    offset_y: f64,
  ) -> Self {
    Self {
      event_name,
      request_id,
      start_millis,
      intensity,
      duration,
      offset_angle_x,
      offset_y,
    }
  }

  pub const fn default_start_millis() -> u64 {
    0
  }

  pub const fn default_intensity() -> f64 {
    1.0
  }

  pub const fn default_duration() -> f64 {
    1.0
  }

  pub const fn default_offset_angle_x() -> f64 {
    0.0
  }

  pub const fn default_offset_y() -> f64 {
    0.0
  }
}

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SdkPlayDotModeMessage {
  request_id: u32,

  #[cfg_attr(
    feature = "serde",
    serde(
      serialize_with = "DevicePosition::serialize_as_repr",
      deserialize_with = "DevicePosition::deserialize_from_repr"
    )
  )]
  position: DevicePosition,

  duration_millis: u64,

  /// Intensity per motor index. Usually these bits are representative of what BLE protocol is sending
  ///
  /// # Values per device position
  ///
  /// ## [DevicePosition::Vest]
  ///
  /// First 20 is front, last 20 is back
  ///
  /// ```text
  /// |    Front    |    Back     |
  /// +-------------+-------------+
  /// | 0   1  2  3 | 20 21 22 23 |
  /// | 4   5  6  7 | 24 25 26 27 |
  /// | 8   9 10 11 | 28 29 30 31 |
  /// | 12 13 14 15 | 32 33 34 35 |
  /// | 16 17 18 19 | 36 37 38 39 | <- this row only for the X40 suit
  /// +-------------+-------------+
  /// ```
  ///
  /// ## [DevicePosition::VestFront], [DevicePosition::VestBack]
  ///
  /// From what I can see it is rarely, if ever, used.
  ///
  /// ```text
  /// +-------------+
  /// | 0   1  2  3 |
  /// | 4   5  6  7 |
  /// | 8   9 10 11 |
  /// | 12 13 14 15 |
  /// | 16 17 18 19 |
  /// +-------------+
  /// ```
  ///
  /// ## [DevicePosition::Head]
  ///
  /// ```text
  /// +-------------------+
  /// |   0   1   2   3   |
  /// | +---------------+ |
  /// | |               | |
  /// | +---+       +---+ |
  /// +-----+       +-----+
  /// ```
  ///
  /// ## [DevicePosition::ForearmL], [DevicePosition::ForearmR]
  ///
  /// ```text
  /// +---------+
  /// |         |
  /// | 0  1  2 |
  /// |         |
  /// +---------+
  /// ```
  ///
  /// ## [DevicePosition::HandL], [DevicePosition::HandR]
  ///
  /// ```text
  /// +-----+
  /// |  0  |
  /// |  1  |
  /// |  2  |
  /// +-----+
  /// ```
  ///
  /// ## [DevicePosition::GloveL], [DevicePosition::GloveR]
  ///
  /// ```text
  ///     .-.
  ///   .-|2|-.
  ///   |1| |3|
  ///   | | | |-.
  ///   | | | |4|
  /// .-| | | | |
  /// |0|     ` |
  /// | |       |
  /// |         |
  /// \         /
  ///  |   5   |
  ///  |       |
  /// ```
  ///
  /// ## [DevicePosition::FootL], [DevicePosition::FootR]
  ///
  /// ```text
  /// +---------+
  /// |         |
  /// | 0  1  2 |
  /// |         |
  /// +---------+
  /// ```
  ///
  /// todo: this is usually [u8; 40] as for 40 motors (values 0-100)
  motor_values: Vec<u8>,
}

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SdkPlayPathModeMessage {
  request_id: u32,

  #[cfg_attr(
    feature = "serde",
    serde(
      serialize_with = "DevicePosition::serialize_as_repr",
      deserialize_with = "DevicePosition::deserialize_from_repr"
    )
  )]
  position: DevicePosition,

  duration_millis: u64,

  path_points: Vec<PathPoint>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[cfg(feature = "serde")]
  #[test]
  fn test_encodes_sdk_request_auth_message_as_string() {
    let msg = SdkMessage::SdkRequestAuth(SdkRequestAuthMessage {
      cipher: "cipher".to_string(),
      application_id: "app_id".to_string(),
      nonce_hash_value: "nonce".to_string(),
      application_id_hash_value: "app_hash".to_string(),
      sdk_api_key: "api_key".to_string(),
    });

    let json = serde_json::to_string(&msg).unwrap();

    let expected = r#"{"type":"SdkRequestAuth","message":"{\"cipher\":\"cipher\",\"applicationId\":\"app_id\",\"nonceHashValue\":\"nonce\",\"applicationIdHashValue\":\"app_hash\",\"sdkApiKey\":\"api_key\"}"}"#;
    assert_eq!(json, expected);
  }

  #[cfg(feature = "serde")]
  #[test]
  fn test_encodes_sdk_request_auth_init_message_as_string() {
    let msg = SdkMessage::SdkRequestAuthInit(SdkRequestAuthInitMessage {
      authentication: SdkRequestAuthMessage {
        cipher: "cipher".to_string(),
        application_id: "app_id".to_string(),
        nonce_hash_value: "nonce".to_string(),
        application_id_hash_value: "app_hash".to_string(),
        sdk_api_key: "api_key".to_string(),
      },
      haptic: SdkApiResponseV3::new(true, 200, None, 1234567890, None),
    });

    let json = serde_json::to_string(&msg).unwrap();

    let expected = r#"{"type":"SdkRequestAuthInit","message":"{\"authentication\":{\"cipher\":\"cipher\",\"applicationId\":\"app_id\",\"nonceHashValue\":\"nonce\",\"applicationIdHashValue\":\"app_hash\",\"sdkApiKey\":\"api_key\"},\"haptic\":{\"status\":true,\"code\":200,\"errorMessage\":null,\"timestamp\":1234567890,\"message\":null}}"}"#;
    assert_eq!(json, expected);
  }

  #[cfg(feature = "serde")]
  #[test]
  fn test_decodes_sdk_request_auth_message_from_string() {
    let json = r#"{"type":"SdkRequestAuth","message":"{\"cipher\":\"cipher\",\"applicationId\":\"app_id\",\"nonceHashValue\":\"nonce\",\"applicationIdHashValue\":\"app_hash\",\"sdkApiKey\":\"api_key\"}"}"#;

    let msg: SdkMessage = serde_json::from_str(json).unwrap();

    let expected = SdkMessage::SdkRequestAuth(SdkRequestAuthMessage {
      cipher: "cipher".to_string(),
      application_id: "app_id".to_string(),
      nonce_hash_value: "nonce".to_string(),
      application_id_hash_value: "app_hash".to_string(),
      sdk_api_key: "api_key".to_string(),
    });

    assert_eq!(msg, expected);
  }
}

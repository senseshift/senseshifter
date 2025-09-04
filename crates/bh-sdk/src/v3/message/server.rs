use derivative::Derivative;
use getset::Getters;
use strum::{EnumDiscriminants, EnumString, VariantNames};

#[derive(Derivative, EnumDiscriminants)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[strum_discriminants(name(ServerMessageType))]
#[strum_discriminants(derive(EnumString, VariantNames))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "message"))]
#[cfg_attr(feature = "serde", serde_with::serde_as)]
pub enum ServerMessage {
    ServerReady,
    ServerEventNameList(
        #[cfg_attr(feature = "serde", serde(with = "serde_handy::as_json_or_object"))] Vec<String>,
    ),
    ServerEventList(
        #[cfg_attr(feature = "serde", serde(with = "serde_handy::as_json_or_object"))]
        Vec<ServerEventListMessageItem>,
    ),
    ServerActiveEventNameList(
        #[cfg_attr(feature = "serde", serde(with = "serde_handy::as_json_or_object"))] Vec<String>,
    ),
    ServerActiveRequestIdList(
        #[cfg_attr(feature = "serde", serde(with = "serde_handy::as_json_or_object"))] Vec<u32>,
    ),
    ServerDevices(
        #[cfg_attr(feature = "serde", serde(with = "serde_handy::as_json_or_object"))]
        Vec<ServerDevicesMessageItem>,
    ),
}

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ServerEventListMessageItem {
    event_name: String,
    event_time: u32,
}

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ServerDevicesMessageItem {
    position: u32,
    device_name: String,
    address: String,
    connected: bool,
    paired: bool,

    /// Battery level (0-100)
    battery: u8,

    audio_jack_in: bool,
    vsm: u32,
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserialize<'de> for ServerMessage {
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

        let tag = ServerMessageType::from_str(tag)
            .map_err(|_| Error::unknown_variant(tag, ServerMessageType::VARIANTS))?;

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
            ServerMessageType::ServerReady => Ok(ServerMessage::ServerReady),
            ServerMessageType::ServerEventNameList => {
                parse_msg(msg_v).map(ServerMessage::ServerEventNameList)
            }
            ServerMessageType::ServerEventList => {
                parse_msg(msg_v).map(ServerMessage::ServerEventList)
            }
            ServerMessageType::ServerActiveEventNameList => {
                parse_msg(msg_v).map(ServerMessage::ServerActiveEventNameList)
            }
            ServerMessageType::ServerActiveRequestIdList => {
                parse_msg(msg_v).map(ServerMessage::ServerActiveRequestIdList)
            }
            ServerMessageType::ServerDevices => parse_msg(msg_v).map(ServerMessage::ServerDevices),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    fn test_decodes_server_devices_message_from_object() {
        let json = r#"{
            "Type":"ServerDevices",
            "Message": [{
                "position":0,
                "deviceName":"TactSuitX40",
                "address":"DF3A9CDC74BB",
                "connected":true,
                "paired":true,
                "battery":98,
                "audioJackIn":false,
                "vsm":20
            },{
                "position":1,"deviceName":
                "Tactosy2_V3 (L)",
                "address":"C9C1A41F2570",
                "connected":true,
                "paired":true,
                "battery":26,
                "audioJackIn":false,
                "vsm":11
            },{
                "position":2,
                "deviceName":"Tactosy2_V3 (R)",
                "address":"FC8A5696C0B8",
                "connected":true,
                "paired":true,
                "battery":62,
                "audioJackIn":false,
                "vsm":11
            }]
        }"#;

        let msg = serde_json::from_str::<ServerMessage>(json);

        assert!(msg.is_ok(), "Failed to parse JSON: {:?}", msg.unwrap_err());
    }
}

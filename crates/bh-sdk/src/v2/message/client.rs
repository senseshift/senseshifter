use bh_haptic_definitions::{HapticFrame, TactFileProject};
use derivative::Derivative;
use getset::Getters;

#[derive(Derivative, Getters)]
#[get = "pub"]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "PascalCase"))]
pub struct ClientMessage {
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    register: Option<Vec<ClientRegisterMessage>>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    submit: Option<Vec<ClientSubmitMessage>>,
}

impl ClientMessage {
    pub fn new(
        register: Option<Vec<ClientRegisterMessage>>,
        submit: Option<Vec<ClientSubmitMessage>>,
    ) -> Self {
        Self { register, submit }
    }

    pub fn new_register(messages: Vec<ClientRegisterMessage>) -> Self {
        Self::new(Some(messages), None)
    }

    pub fn new_submit(messages: Vec<ClientSubmitMessage>) -> Self {
        Self::new(None, Some(messages))
    }
}

#[derive(Derivative, Getters)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[get = "pub"]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientRegisterMessage {
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "Key",
            deserialize_with = "serde_handy::de::from_str_num_to_string"
        )
    )]
    key: String,

    #[cfg_attr(feature = "serde", serde(alias = "Project"))]
    project: TactFileProject,
}

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "PascalCase", tag = "Type"))]
pub enum ClientSubmitMessage {
    #[cfg_attr(feature = "serde", serde(rename = "turnOffAll"))]
    TurnOffAll,

    #[cfg_attr(
        feature = "serde",
        serde(rename = "turnOff", rename_all = "PascalCase")
    )]
    TurnOff {
        #[cfg_attr(
            feature = "serde",
            serde(
                rename = "Key",
                deserialize_with = "serde_handy::de::from_str_num_to_string"
            )
        )]
        key: String,
    },

    #[cfg_attr(feature = "serde", serde(rename = "key", rename_all = "PascalCase"))]
    Key {
        #[cfg_attr(
            feature = "serde",
            serde(
                rename = "Key",
                deserialize_with = "serde_handy::de::from_str_num_to_string"
            )
        )]
        key: String,
    },

    #[cfg_attr(feature = "serde", serde(rename = "frame", rename_all = "PascalCase"))]
    Frame {
        #[cfg_attr(
            feature = "serde",
            serde(
                rename = "Key",
                deserialize_with = "serde_handy::de::from_str_num_to_string"
            )
        )]
        key: String,

        frame: HapticFrame,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize_message() {
        assert_eq!(
            serde_json::from_str::<ClientMessage>(r#"{"Register": []}"#).unwrap(),
            ClientMessage::new_register(vec![])
        );
        assert_eq!(
            serde_json::from_str::<ClientMessage>(
                r#"{"Submit": [{"Type": "turnOff", "Key": "example_key"}]}"#
            )
            .unwrap(),
            ClientMessage::new_submit(vec![ClientSubmitMessage::TurnOff {
                key: "example_key".to_string()
            }])
        );
        assert_eq!(
            serde_json::from_str::<ClientMessage>(r#"{"Submit": [{"Type": "turnOffAll"}]}"#)
                .unwrap(),
            ClientMessage::new_submit(vec![ClientSubmitMessage::TurnOffAll])
        );
    }
}

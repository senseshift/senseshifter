mod common;

use common::*;
use std::fs::read_to_string;

use derivative::Derivative;

#[cfg(feature = "v2")]
use bh_sdk::v2::{ClientMessage, ServerMessage};

#[cfg(feature = "v3")]
use bh_sdk::v3::SdkMessage;

#[cfg(feature = "v2")]
#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
enum SdkV2Message {
    Client(ClientMessage),
    Server(ServerMessage),
}

#[cfg(all(feature = "serde", feature = "v2"))]
#[test]
fn test_deserialize_v2_stream() -> anyhow::Result<()> {
    let dir = fixture_path("v2").join("valid");

    for entry in walkdir::WalkDir::new(&dir) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();

        let name = path.file_name().unwrap().to_str().unwrap();

        for (_i, line) in read_to_string(path)?.lines().enumerate() {
            if line.trim().is_empty() || line.starts_with("//") || line.starts_with('#') {
                continue;
            }

            let parsed = serde_json::from_str::<SdkV2Message>(line);

            assert!(
                parsed.is_ok(),
                "Failed to parse {}: {:?}",
                name,
                parsed.unwrap_err()
            );
        }
    }

    Ok(())
}

#[cfg(feature = "v3")]
#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
enum SdkV3Message {
    SdkMessage(SdkMessage),
}

#[cfg(all(feature = "serde", feature = "v2"))]
#[test]
fn test_deserialize_v3_stream() -> anyhow::Result<()> {
    let dir = fixture_path("v3").join("valid");

    for entry in walkdir::WalkDir::new(&dir) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();

        let name = path.file_name().unwrap().to_str().unwrap();

        for (_i, line) in read_to_string(path)?.lines().enumerate() {
            if line.trim().is_empty() || line.starts_with("//") || line.starts_with('#') {
                continue;
            }

            let parsed = serde_json::from_str::<SdkMessage>(line);

            assert!(
                parsed.is_ok(),
                "Failed to parse {}: {:?}",
                name,
                parsed.unwrap_err()
            );
        }
    }

    Ok(())
}

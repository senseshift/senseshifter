mod common;

use std::fs::read_to_string;
use common::*;

use derivative::Derivative;

use bh_sdk::v2::{ClientMessage, ServerMessage};

#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
enum SdkV2Message {
    Client(ClientMessage),
    Server(ServerMessage),
}

#[cfg(feature = "serde")]
#[test]
fn test_deserialize_v2_stream() -> anyhow::Result<()> {
    let dir = fixture_path("v2").join("valid");

    for entry in walkdir::WalkDir::new(&dir) {
        let entry = entry?;
        if !entry.file_type().is_file() { continue; }
        let path = entry.path();

        let name = path.file_name().unwrap().to_str().unwrap();

        for (_i, line) in read_to_string(path)?.lines().enumerate() {
            let parsed = serde_json::from_str::<SdkV2Message>(line);

            assert!(parsed.is_ok(), "Failed to parse {}: {:?}", name, parsed.unwrap_err());
        }
    }

    Ok(())
}

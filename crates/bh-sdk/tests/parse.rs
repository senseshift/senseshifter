mod common;

use common::*;
use std::fs::read_to_string;

use derivative::Derivative;

#[cfg(feature = "v2")]
use bh_sdk::v2::{ClientMessage as ClientMessageV2, ServerMessage as ServerMessageV2};

#[cfg(feature = "v3")]
use bh_sdk::v3::{SdkMessage as SdkMessageV3, ServerMessage as ServerMessageV3};

#[cfg(feature = "v2")]
#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[allow(clippy::large_enum_variant)] // this is a test enum, not a real enum
enum SdkV2Message {
  Client(ClientMessageV2),
  Server(ServerMessageV2),
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

    for line in read_to_string(path)?.lines() {
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
struct SdkV3Message {
  sdk_message: serde_json::Result<SdkMessageV3>,
  server_message: serde_json::Result<ServerMessageV3>,
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

    for line in read_to_string(path)?.lines() {
      if line.trim().is_empty() || line.starts_with("//") || line.starts_with('#') {
        continue;
      }

      let parsed = SdkV3Message {
        sdk_message: serde_json::from_str::<SdkMessageV3>(line),
        server_message: serde_json::from_str::<ServerMessageV3>(line),
      };

      assert!(
        parsed.sdk_message.is_ok() || parsed.server_message.is_ok(),
        "Failed to parse {}: {:?}",
        name,
        (parsed.sdk_message.err(), parsed.server_message.err()),
      );
    }
  }

  Ok(())
}

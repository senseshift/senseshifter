use bh_haptic_definitions::{HapticDefinitionsMessage, SdkApiResponseV3, TactFile};

mod common;

#[test]
fn haptic_definitions_parse_valid() -> anyhow::Result<()>  {
  let dir = common::fixture_path("haptic_definitions").join("valid");

  for entry in walkdir::WalkDir::new(&dir) {
    let entry = entry?;
    if !entry.file_type().is_file() { continue; }
    let path = entry.path();
    let name = path.file_name().unwrap().to_str().unwrap();
    let data = std::fs::read_to_string(path)?;

    let parsed = serde_json::from_str::<SdkApiResponseV3<HapticDefinitionsMessage>>(&data);

    assert!(parsed.is_ok(), "Failed to parse {}: {:?}", name, parsed);

    let parsed = parsed?;

    let expected_value = serde_json::from_str::<serde_json::Value>(&data).unwrap();
    let parsed_value = serde_json::to_value(parsed).unwrap();

    // assert_eq!(expected_value, parsed_value);
  }

  Ok(())
}

#[test]
fn tact_files_parse_valid() -> anyhow::Result<()>  {
  let dir = common::fixture_path("tact_file").join("valid");

  for entry in walkdir::WalkDir::new(&dir) {
    let entry = entry?;
    if !entry.file_type().is_file() { continue; }
    let path = entry.path();
    let name = path.file_name().unwrap().to_str().unwrap();
    let data = std::fs::read_to_string(path)?;

    let parsed = serde_json::from_str::<TactFile>(&data);

    assert!(parsed.is_ok(), "Failed to parse {}: {:?}", name, parsed);
  }

  Ok(())
}
use ss_bh::server::haptic_definitions::HapticDefinitionsResponse;

mod common;

#[test]
fn parses_all_valid_fixtures() -> anyhow::Result<()> {
  let dir = common::fixture_path("haptic_definitions").join("valid");

  for entry in walkdir::WalkDir::new(&dir) {
    let entry = entry?;
    if !entry.file_type().is_file() { continue; }
    let path = entry.path();
    let name = path.file_name().unwrap().to_str().unwrap();
    let data = std::fs::read_to_string(path)?;

    let parsed = serde_json::from_str::<HapticDefinitionsResponse>(&data);

    assert!(parsed.is_ok(), "Failed to parse {}: {:?}", name, parsed);

    let parsed = parsed?;

    let expected_value = serde_json::from_str::<serde_json::Value>(&data).unwrap();
    let parsed_value = serde_json::to_value(parsed).unwrap();

    assert_eq!(expected_value, parsed_value);
  }

  Ok(())
}
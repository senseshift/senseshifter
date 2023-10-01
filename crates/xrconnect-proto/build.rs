fn main() -> Result<(), Box<dyn std::error::Error>> {
  let out_dir = "src/codegen";

  let source_files = [
    "xrconnect/devices/v1alpha1/devicemanager.proto"
  ];

  let builder = tonic_build::configure()
    .out_dir(out_dir)
    .type_attribute(
      ".",
      "#[cfg_attr(feature = \"serde\", derive(::serde::Serialize, ::serde::Deserialize), serde(rename_all = \"snake_case\"))]",
    )
    .type_attribute(
      ".",
      "#[cfg_attr(feature = \"specta\", derive(::specta::Type))]",
    )
    .server_mod_attribute(".", "#[cfg(feature = \"tonic-server\")]")
    .client_mod_attribute(".", "#[cfg(feature = \"tonic-client\")]");

  builder
    .compile(
      &source_files,
      &["proto"],
    )
    .unwrap();

  let replacements = [
    ("open_glove", "openglove"),
    ("Openglove", "OpenGlove"),
    ("BluetoothLe", "BluetoothLE"),
    ("Ble", "BLE"),
    ("Rfcomm", "RFComm"),
    ("RfComm", "RFComm")
  ];

  // files in out_dir with '.rs' extension
  let filenames = std::fs::read_dir(out_dir)?
    .filter(|entry| {
      let entry = match entry {
        Ok(entry) => entry,
        Err(_) => return false,
      };
      let path = entry.path();

      path.is_file() && path.extension().map(|ext| ext == "rs").unwrap_or(false)
    });

  for filename in filenames {
    let filename = filename?.path();
    let mut contents = std::fs::read_to_string(&filename)?;

    for (from, to) in &replacements {
      contents = contents.replace(from, to);
    }

    std::fs::write(&filename, contents)?;
  }

  Ok(())
}
use std::path::{Path, PathBuf};

pub fn fixtures_dir() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests")
    .join("fixtures")
}

pub fn fixture_path<P: AsRef<Path>>(rel: P) -> PathBuf {
  fixtures_dir().join(rel)
}

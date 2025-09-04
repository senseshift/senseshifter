pub mod de;

#[cfg(feature = "json")]
pub mod as_json_or_object {
  use serde::de::{DeserializeOwned, Error as DeError};
  use serde::ser::Error as SerError;
  use serde::{Deserialize, Deserializer, Serialize, Serializer};
  use serde_json as sj;

  // Serialize the payload as a JSON-encoded string
  pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
  where
    T: Serialize,
    S: Serializer,
  {
    let s = sj::to_string(value).map_err(S::Error::custom)?;
    serializer.serialize_str(&s)
  }

  // Deserialize from either stringified JSON or a plain object
  pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
  where
    T: DeserializeOwned, // <-- key change
    D: Deserializer<'de>,
  {
    let v = sj::Value::deserialize(deserializer)?;
    match v {
      sj::Value::String(s) => sj::from_str::<T>(&s).map_err(D::Error::custom),
      other => sj::from_value::<T>(other).map_err(D::Error::custom),
    }
  }
}

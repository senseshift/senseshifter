use serde::de::{self, Deserializer, Unexpected, Visitor};
use std::fmt;
use serde::Deserialize;

#[derive(Debug)]
struct StrNum(String);

impl From<StrNum> for String {
  fn from(s: StrNum) -> Self {
    s.0
  }
}

impl<'de> serde::Deserialize<'de> for StrNum {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct V;

    impl<'de> Visitor<'de> for V {
      type Value = StrNum;

      fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a string or a number")
      }

      // strings
      fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Ok(StrNum(v.to_owned()))
      }
      fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Ok(StrNum(v.to_owned()))
      }
      fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Ok(StrNum(v))
      }

      // numbers
      fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Ok(StrNum(v.to_string()))
      }
      fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Ok(StrNum(v.to_string()))
      }
      fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Ok(StrNum(v.to_string()))
      }

      // everything else -> error (adjust if desired)
      fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Err(E::invalid_type(Unexpected::Bool(v), &"string or number"))
      }
      fn visit_unit<E>(self) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Err(E::invalid_type(Unexpected::Unit, &"string or number"))
      }
    }

    deserializer.deserialize_any(V)
  }
}

/// For `String` fields.
pub fn from_str_num_to_string<'de, D>(d: D) -> Result<String, D::Error>
where
  D: Deserializer<'de>,
{
  StrNum::deserialize(d).map(Into::into)
}

/// For `Option<String>` fields (handles null/None/missing via `#[serde(default)]`).
pub fn from_str_num_to_opt_string<'de, D>(d: D) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  Option::<StrNum>::deserialize(d).map(|o| o.map(Into::into))
}

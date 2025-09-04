use serde::Deserialize;
use serde::de::{self, Deserializer, Unexpected, Visitor};
use std::{fmt, marker::PhantomData};

/// Trait to abstract over f32/f64 without duplication.
trait FloatParse: Sized + 'static {
  const NAME: &'static str;
  fn from_f64(v: f64) -> Self;
  fn parse_str(s: &str) -> Result<Self, std::num::ParseFloatError>;
}

impl FloatParse for f32 {
  const NAME: &'static str = "f32";
  fn from_f64(v: f64) -> Self {
    v as f32
  }
  fn parse_str(s: &str) -> Result<Self, std::num::ParseFloatError> {
    s.parse()
  }
}

impl FloatParse for f64 {
  const NAME: &'static str = "f64";
  fn from_f64(v: f64) -> Self {
    v
  }
  fn parse_str(s: &str) -> Result<Self, std::num::ParseFloatError> {
    s.parse()
  }
}

/// Core wrapper: accepts either a number (i64/u64/f64) or a string, producing T.
#[derive(Debug)]
struct FromStrOrNum<T>(T);

impl<'de, T> Deserialize<'de> for FromStrOrNum<T>
where
  T: FloatParse,
{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct V<T>(PhantomData<T>);
    impl<'de, T> Visitor<'de> for V<T>
    where
      T: FloatParse,
    {
      type Value = FromStrOrNum<T>;

      fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a string or a number parseable as {}", T::NAME)
      }

      // Numbers (Serde’s generic model)
      fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Ok(FromStrOrNum(T::from_f64(v)))
      }
      fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Ok(FromStrOrNum(T::from_f64(v as f64)))
      }
      fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Ok(FromStrOrNum(T::from_f64(v as f64)))
      }

      // Strings
      fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        let s = v.trim();
        T::parse_str(s)
          .map(FromStrOrNum)
          .map_err(|e| E::custom(format!("invalid {} string `{}`: {}", T::NAME, s, e)))
      }
      fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        self.visit_borrowed_str(v)
      }
      fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        self.visit_borrowed_str(&v)
      }

      // Everything else → error (tweak if you want to accept more)
      fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Err(E::invalid_type(Unexpected::Bool(v), &self))
      }
      fn visit_unit<E>(self) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Err(E::invalid_type(Unexpected::Unit, &self))
      }
      fn visit_none<E>(self) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Err(E::invalid_type(Unexpected::Option, &self))
      }
    }

    deserializer.deserialize_any(V::<T>(PhantomData))
  }
}

/// Public helpers — thin adapters with no duplicated logic.
pub fn to_f32<'de, D>(d: D) -> Result<f32, D::Error>
where
  D: Deserializer<'de>,
{
  FromStrOrNum::<f32>::deserialize(d).map(|v| v.0)
}

pub fn to_f64<'de, D>(d: D) -> Result<f64, D::Error>
where
  D: Deserializer<'de>,
{
  FromStrOrNum::<f64>::deserialize(d).map(|v| v.0)
}

pub fn to_opt_f32<'de, D>(d: D) -> Result<Option<f32>, D::Error>
where
  D: Deserializer<'de>,
{
  Option::<FromStrOrNum<f32>>::deserialize(d).map(|o| o.map(|v| v.0))
}

pub fn to_opt_f64<'de, D>(d: D) -> Result<Option<f64>, D::Error>
where
  D: Deserializer<'de>,
{
  Option::<FromStrOrNum<f64>>::deserialize(d).map(|o| o.map(|v| v.0))
}

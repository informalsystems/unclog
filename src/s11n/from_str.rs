//! Adapters for serializing/deserializing types that implement `FromStr` and
//! `std::fmt::Display`.

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

/// Serialize `value.to_string()`
pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: std::fmt::Display,
{
    value.to_string().serialize(serializer)
}

/// Deserialize a string and attempt to parse it into an instance of type `T`.
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    String::deserialize(deserializer)?
        .parse::<T>()
        .map_err(|e| D::Error::custom(format!("{e}")))
}

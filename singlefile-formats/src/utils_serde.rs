#![cfg_attr(docsrs, doc(cfg(feature = "utils-serde")))]
#![cfg(feature = "utils-serde")]

//! Utilities for use with [`serde`].

use singlefile::{FileFormat, FileFormatUtf8};
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};



/// An interface which may be used to define types which can be used in
/// `serde_derive`'s `#[serde(with = "...")]` attributes to serialize and
/// deserialize [`FileFormat`]-compatible types.
///
/// Types will be converted to/from a `Vec<u8>` and serialized/deserialized that way.
pub trait FormatAdapter<T> {
  /// The type of the [`FileFormat`].
  type Format: FileFormat<T>;

  /// The [`FileFormat`] that will be used for serializing and deserializing values.
  const FORMAT: Self::Format;

  /// Serializes the given value using the adapter's designated [`FileFormat`].
  ///
  /// This function may be used in `serde_derive`'s `#[serde(serialize_with = "...")]` attribute.
  fn serialize<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer {
    Self::FORMAT.to_buffer(value)
      .map_err(serde::ser::Error::custom)
      .and_then(|buf| buf.serialize(serializer))
  }

  /// Deserializes a value using the adapter's designated [`FileFormat`].
  ///
  /// This function may be used in `serde_derive`'s `#[serde(deserialize_with = "...")]` attribute.
  fn deserialize<'de, D>(deserializer: D) -> Result<T, D::Error>
  where D: Deserializer<'de> {
    <Vec<u8>>::deserialize(deserializer).and_then(|buf| {
      Self::FORMAT.from_buffer(&buf).map_err(serde::de::Error::custom)
    })
  }
}

/// An interface which may be used to define types which can be used in
/// `serde_derive`'s `#[serde(with = "...")]` attributes to serialize and
/// deserialize [`FileFormat`]-compatible types.
///
/// Types will be converted to/from a `String` and serialized/deserialized that way.
pub trait FormatAdapterText<T> {
  /// The type of the [`FileFormatUtf8`].
  type Format: FileFormatUtf8<T>;

  /// The [`FileFormatUtf8`] that will be used for serializing and deserializing values.
  const FORMAT: Self::Format;

  /// Serializes the given value using the adapter's designated [`FileFormatUtf8`].
  ///
  /// This function may be used in `serde_derive`'s `#[serde(serialize_with = "...")]` attribute.
  fn serialize<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer {
    Self::FORMAT.to_string_buffer(value)
      .map_err(serde::ser::Error::custom)
      .and_then(|buf| buf.serialize(serializer))
  }

  /// Deserializes a value using the adapter's designated [`FileFormatUtf8`].
  ///
  /// This function may be used in `serde_derive`'s `#[serde(deserialize_with = "...")]` attribute.
  fn deserialize<'de, D>(deserializer: D) -> Result<T, D::Error>
  where D: Deserializer<'de> {
    <String>::deserialize(deserializer).and_then(|buf| {
      Self::FORMAT.from_string_buffer(&buf).map_err(serde::de::Error::custom)
    })
  }
}

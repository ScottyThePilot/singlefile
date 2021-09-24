use serde::ser::Serialize;
use serde::de::DeserializeOwned;

use std::io::{Read, Write};



/// An error created by an implementation of `FileFormat`.
pub type FormatError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Describes how a format can be written to or read from a stream.
///
/// # Example
/// ```rust
/// struct Json;
///
/// impl FileFormat for Json {
///   #[inline]
///   fn to_writer<W, T>(&self, writer: W, value: &T) -> Result<(), FormatError>
///   where W: Write, T: Serialize {
///     serde_json::to_writer_pretty(writer, value).map_err(From::from)
///   }
///
///   #[inline]
///   fn from_reader<R, T>(&self, reader: R) -> Result<T, FormatError>
///   where R: Read, T: DeserializeOwned {
///     serde_json::from_reader(reader).map_err(From::from)
///   }
/// }
/// ```
pub trait FileFormat {
  /// Serialize a value into a `Write` stream.
  fn to_writer<W, T>(&self, writer: W, value: &T) -> Result<(), FormatError>
  where W: Write, T: Serialize;

  /// Deserialize a value from a `Read` stream.
  fn from_reader<R, T>(&self, reader: R) -> Result<T, FormatError>
  where R: Read, T: DeserializeOwned;
}

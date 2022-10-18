//! How to interpret the contents of files.

pub mod default_formats;

pub use self::default_formats::PlainBytes;
pub use self::default_formats::PlainUtf8;

use std::io::{Cursor, Read, Write};

/// A trait that describes how a file's contents should be interpreted.
///
/// Usually, you will want to implement a simple wrapper over your file format's
/// `to_writer` and `from_reader` functions, using your favorite serialization framework.
///
/// # Example
/// ```no_run
/// # use serde::ser::Serialize;
/// # use serde::de::DeserializeOwned;
/// # use singlefile::FileFormat;
/// # use std::io::{Read, Write};
/// struct Json;
///
/// impl<T> FileFormat<T> for Json
/// where T: Serialize + DeserializeOwned {
///   type FormatError = serde_json::Error;
///
///   fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
///     serde_json::to_writer_pretty(writer, value).map_err(From::from)
///   }
///
///   fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
///     serde_json::from_reader(reader).map_err(From::from)
///   }
/// }
/// ```
pub trait FileFormat<T> {
  /// The type of error to return from `to_writer` and `from_reader`.
  type FormatError: std::error::Error;

  /// Serialize a value into a `Write` stream.
  fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError>;

  /// Serialize a value into a byte vec.
  fn to_buf(&self, value: &T) -> Result<Vec<u8>, Self::FormatError> {
    let mut buf = Cursor::new(Vec::new());
    self.to_writer(&mut buf, value)?;
    Ok(buf.into_inner())
  }

  /// Deserialize a value from a `Read` stream.
  fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError>;
}

impl<T, Format> FileFormat<T> for &Format
where Format: FileFormat<T> {
  type FormatError = <Format as FileFormat<T>>::FormatError;

  #[inline]
  fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
    Format::to_writer(self, writer, value)
  }

  #[inline]
  fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    Format::from_reader(self, reader)
  }
}

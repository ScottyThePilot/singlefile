//! How to interpret the contents of files.

pub mod default_formats;

pub use self::default_formats::PlainBytes;
pub use self::default_formats::PlainUtf8;

use std::io::{Cursor, BufReader, BufWriter, Read, Write};

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
/// # use singlefile_formats::data::json_serde::original as serde_json;
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

  /// Deserialize a value from a `Read` stream.
  ///
  /// If you are reading directly from a [`File`][std::fs::File], you should consider
  /// using [`from_reader_buffered`][FileFormat::from_reader_buffered] instead.
  fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError>;

  /// Identical to [`FileFormat::from_reader`], however the provided reader is buffered with [`BufReader`].
  ///
  /// You should override this function if your file format reads
  /// to a buffer internally in order to avoid double-buffering.
  #[inline]
  fn from_reader_buffered<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    self.from_reader(BufReader::new(reader))
  }

  /// Deserialize a value from a byte vec.
  #[inline]
  fn from_buffer(&self, buf: &[u8]) -> Result<T, Self::FormatError> {
    self.from_reader(buf)
  }

  /// Serialize a value into a `Write` stream.
  ///
  /// If you are writing directly to a [`File`][std::fs::File], you should consider
  /// using [`to_writer_buffered`][FileFormat::to_writer_buffered] instead.
  fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError>;

  /// Identical to [`FileFormat::to_writer`], however the provided writer is buffered with [`BufWriter`].
  ///
  /// You should override this function if your file format writes
  /// to a buffer internally in order to avoid double-buffering.
  #[inline]
  fn to_writer_buffered<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
    self.to_writer(BufWriter::new(writer), value)
  }

  /// Serialize a value into a byte vec.
  fn to_buffer(&self, value: &T) -> Result<Vec<u8>, Self::FormatError> {
    let mut buf = Cursor::new(Vec::new());
    self.to_writer(&mut buf, value)?;
    Ok(buf.into_inner())
  }
}

/// A trait that indicates a file's contents will always be valid UTF-8.
pub trait FileFormatUtf8<T>: FileFormat<T> {
  /// Deserialize a buffer from a string slice.
  fn from_string_buffer(&self, buf: &str) -> Result<T, Self::FormatError>;

  /// Serialize a value into a string buffer.
  fn to_string_buffer(&self, value: &T) -> Result<String, Self::FormatError>;
}

macro_rules! impl_file_format_delegate {
  (<$Format:ident> $Type:ty) => (
    impl<T, $Format: FileFormat<T>> FileFormat<T> for $Type {
      type FormatError = <$Format as FileFormat<T>>::FormatError;

      #[inline]
      fn from_reader_buffered<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
        $Format::from_reader_buffered(self, reader)
      }

      #[inline]
      fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
        $Format::from_reader(self, reader)
      }

      #[inline]
      fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
        $Format::to_writer(self, writer, value)
      }

      #[inline]
      fn to_writer_buffered<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
        $Format::to_writer_buffered(self, writer, value)
      }

      #[inline]
      fn to_buffer(&self, value: &T) -> Result<Vec<u8>, Self::FormatError> {
        $Format::to_buffer(self, value)
      }
    }

    impl<T, $Format: FileFormatUtf8<T>> FileFormatUtf8<T> for $Type {
      fn from_string_buffer(&self, buf: &str) -> Result<T, Self::FormatError> {
        $Format::from_string_buffer(self, buf)
      }

      fn to_string_buffer(&self, value: &T) -> Result<String, Self::FormatError> {
        $Format::to_string_buffer(self, value)
      }
    }
  );
}

impl_file_format_delegate!(<Format> &Format);
impl_file_format_delegate!(<Format> std::boxed::Box<Format>);
impl_file_format_delegate!(<Format> std::rc::Rc<Format>);
impl_file_format_delegate!(<Format> std::sync::Arc<Format>);

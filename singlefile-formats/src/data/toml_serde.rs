#![cfg_attr(docsrs, doc(cfg(feature = "toml-serde")))]
#![cfg(feature = "toml-serde")]

//! Defines a [`FileFormat`] using the TOML data format.

pub extern crate toml as original;

use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use singlefile::{FileFormat, FileFormatUtf8};
use thiserror::Error;

use std::io::{Read, Write};

/// An error that can occur while using [`Toml`].
#[derive(Debug, Error)]
pub enum TomlError {
  /// An error occured while reading data to the string buffer.
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  /// An error occurred while serializing.
  #[error(transparent)]
  SerializeError(#[from] toml::ser::Error),
  /// An error occurred while deserializing.
  #[error(transparent)]
  DeserializeError(#[from] toml::de::Error)
}

/// A [`FileFormat`] corresponding to the TOML data format.
/// Implemented using the [`toml`] crate, only compatible with [`serde`] types.
///
/// This type provides an optional constant generic parameter for configuring pretty-print.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Toml<const PRETTY: bool = true>;

/// Since the [`toml`] crate exposes no writer-based operations, all operations within this implementation are buffered.
impl<T, const PRETTY: bool> FileFormat<T> for Toml<PRETTY>
where T: Serialize + DeserializeOwned {
  type FormatError = TomlError;

  fn from_reader<R: Read>(&self, mut reader: R) -> Result<T, Self::FormatError> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    toml::de::from_str(&buf).map_err(From::from)
  }

  #[inline]
  fn from_reader_buffered<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    // no need to pass `reader` in with a `BufReader` as that would cause things to be buffered twice
    self.from_reader(reader)
  }

  fn to_writer<W: Write>(&self, mut writer: W, value: &T) -> Result<(), Self::FormatError> {
    let buf = self.to_buffer(value)?;
    writer.write_all(&buf).map_err(From::from)
  }

  #[inline]
  fn to_writer_buffered<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
    // no need to pass `writer` in with a `BufWriter` as that would cause things to be buffered twice
    self.to_writer(writer, value)
  }

  #[inline]
  fn to_buffer(&self, value: &T) -> Result<Vec<u8>, Self::FormatError> {
    self.to_string_buffer(value).map(String::into_bytes)
  }
}

impl<T, const PRETTY: bool> FileFormatUtf8<T> for Toml<PRETTY>
where T: Serialize + DeserializeOwned {
  fn from_string_buffer(&self, buf: &str) -> Result<T, Self::FormatError> {
    Ok(toml::de::from_str(buf)?)
  }

  fn to_string_buffer(&self, value: &T) -> Result<String, Self::FormatError> {
    Ok(match PRETTY {
      true => toml::ser::to_string_pretty(value),
      false => toml::ser::to_string(value)
    }?)
  }
}

/// A shortcut type to a [`Toml`] with pretty-print enabled.
pub type PrettyToml = Toml<true>;
/// A shortcut type to a [`Toml`] with pretty-print disabled.
pub type RegularToml = Toml<false>;

/// A shortcut type to a [`Compressed`][crate::compression::Compressed] [`Toml`].
/// Provides parameters for compression format and pretty-print configuration (defaulting to off).
#[cfg_attr(docsrs, doc(cfg(feature = "compression")))]
#[cfg(feature = "compression")]
pub type CompressedToml<C, const PRETTY: bool = false> = crate::compression::Compressed<C, Toml<PRETTY>>;

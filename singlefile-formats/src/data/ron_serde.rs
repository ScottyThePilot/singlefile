#![cfg_attr(docsrs, doc(cfg(feature = "ron-serde")))]
#![cfg(feature = "ron-serde")]

//! Defines a [`FileFormat`] using the RON data format.

pub extern crate ron as original;

use ron::options::Options;
use ron::ser::PrettyConfig;
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use singlefile::{FileFormat, FileFormatUtf8};

use std::io::{Read, Write};

/// An error that can occur while using [`Ron`] or [`RonPretty`].
pub type RonError = ron::error::Error;

/// A [`FileFormat`] corresponding to the JSON data format.
/// Implemented using the [`ron`] crate, only compatible with [`serde`] types.
///
/// This will not pretty-print code when serializing.
#[derive(Debug, Clone, Default)]
pub struct Ron {
  /// The [`Options`] to use when serializing or deserializing.
  pub options: Options
}

impl<T> FileFormat<T> for Ron
where T: Serialize + DeserializeOwned {
  type FormatError = RonError;

  fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    self.options.from_reader(reader).map_err(Into::into)
  }

  fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
    self.options.to_io_writer(writer, value)
  }

  fn from_buffer(&self, buf: &[u8]) -> Result<T, Self::FormatError> {
    self.options.from_bytes(buf).map_err(Into::into)
  }

  fn to_buffer(&self, value: &T) -> Result<Vec<u8>, Self::FormatError> {
    self.to_string_buffer(value).map(String::into_bytes)
  }
}

impl<T> FileFormatUtf8<T> for Ron
where T: Serialize + DeserializeOwned {
  fn from_string_buffer(&self, buf: &str) -> Result<T, Self::FormatError> {
    self.options.from_str(buf).map_err(Into::into)
  }

  fn to_string_buffer(&self, value: &T) -> Result<String, Self::FormatError> {
    self.options.to_string(value)
  }
}

/// A [`FileFormat`] corresponding to the JSON data format.
/// Implemented using the [`ron`] crate, only compatible with [`serde`] types.
///
/// This will pretty-print code when serializing, using the provided pretty-print config.
#[derive(Debug, Clone, Default)]
pub struct RonPretty {
  /// The [`Options`] to use when serializing or deserializing.
  pub options: Options,
  /// The [`PrettyConfig`] to use when serializing.
  pub config: PrettyConfig
}

impl<T> FileFormat<T> for RonPretty
where T: Serialize + DeserializeOwned {
  type FormatError = RonError;

  fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    self.options.from_reader(reader).map_err(Into::into)
  }

  fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
    self.options.to_io_writer_pretty(writer, value, self.config.clone())
  }

  fn from_buffer(&self, buf: &[u8]) -> Result<T, Self::FormatError> {
    self.options.from_bytes(buf).map_err(Into::into)
  }

  fn to_buffer(&self, value: &T) -> Result<Vec<u8>, Self::FormatError> {
    self.to_string_buffer(value).map(String::into_bytes)
  }
}

impl<T> FileFormatUtf8<T> for RonPretty
where T: Serialize + DeserializeOwned {
  fn from_string_buffer(&self, buf: &str) -> Result<T, Self::FormatError> {
    self.options.from_str(buf).map_err(Into::into)
  }

  fn to_string_buffer(&self, value: &T) -> Result<String, Self::FormatError> {
    self.options.to_string_pretty(value, self.config.clone())
  }
}

/// A shortcut type to a [`Compressed`][crate::compression::Compressed] [`Ron`].
///
/// This will not pretty-print code when serializing.
///
/// Provides a single parameter for compression format.
#[cfg_attr(docsrs, doc(cfg(feature = "compression")))]
#[cfg(feature = "compression")]
pub type CompressedRon<C> = crate::compression::Compressed<C, Ron>;

/// A shortcut type to a [`Compressed`][crate::compression::Compressed] [`Ron`].
///
/// This will pretty-print code when serializing, so consider using [`CompressedRon`] instead.
///
/// Provides a single parameter for compression format.
#[cfg_attr(docsrs, doc(cfg(feature = "compression")))]
#[cfg(feature = "compression")]
pub type CompressedRonPretty<C> = crate::compression::Compressed<C, RonPretty>;

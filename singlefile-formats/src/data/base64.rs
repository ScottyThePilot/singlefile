#![cfg_attr(docsrs, doc(cfg(feature = "base64")))]
#![cfg(feature = "base64")]

//! Defines a [`FileFormat`] that wraps data from another format in Base64.

pub extern crate base64 as original;

use base64::engine::Engine;
use base64::engine::general_purpose::*;
use base64::read::DecoderReader;
use base64::write::{EncoderWriter, EncoderStringWriter};
use singlefile::{FileFormat, FileFormatUtf8};

use std::io::{Read, Write};

/// Takes a [`FileFormat`], encoding any the contents emitted by the format in Base64 before
/// writing to disk, and decoding contents emitted by the format from Base64 before parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Base64<F, E = GeneralPurpose> {
  /// The [`FileFormat`] to be used.
  pub format: F,
  /// The [`Engine`] to be used.
  pub engine: E
}

impl<F, E> Base64<F, E> where E: Engine {
  /// Creates a new [`Base64`], given an engine to encode and decode with.
  pub const fn new(format: F, engine: E) -> Self {
    Base64 { format, engine }
  }
}

impl<F> Base64<F, GeneralPurpose> {
  /// Creates a [`Base64`] using the [`STANDARD`] engine.
  pub const fn with_standard(format: F) -> Self {
    Self::new(format, STANDARD)
  }

  /// Creates a [`Base64`] using the [`STANDARD_NO_PAD`] engine.
  pub const fn with_standard_no_pad(format: F) -> Self {
    Self::new(format, STANDARD_NO_PAD)
  }

  /// Creates a [`Base64`] using the [`URL_SAFE`] engine.
  pub const fn with_url_safe(format: F) -> Self {
    Self::new(format, URL_SAFE)
  }

  /// Creates a [`Base64`] using the [`URL_SAFE_NO_PAD`] engine.
  pub const fn with_url_safe_no_pad(format: F) -> Self {
    Self::new(format, URL_SAFE_NO_PAD)
  }
}

impl<F, E> Default for Base64<F, E> where F: Default, E: Default {
  fn default() -> Self {
    Base64 { format: F::default(), engine: E::default() }
  }
}

impl<F, E, T> FileFormat<T> for Base64<F, E>
where F: FileFormat<T>, E: Engine {
  type FormatError = F::FormatError;

  fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    self.format.from_reader(DecoderReader::new(reader, &self.engine))
  }

  fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
    self.format.to_writer(EncoderWriter::new(writer, &self.engine), value)
  }
}

impl<F, E, T> FileFormatUtf8<T> for Base64<F, E>
where F: FileFormat<T>, E: Engine {
  fn from_string_buffer(&self, buf: &str) -> Result<T, Self::FormatError> {
    self.from_buffer(buf.as_bytes())
  }

  fn to_string_buffer(&self, value: &T) -> Result<String, Self::FormatError> {
    let mut writer = EncoderStringWriter::new(&self.engine);
    self.format.to_writer(&mut writer, value)?;
    Ok(writer.into_inner())
  }
}

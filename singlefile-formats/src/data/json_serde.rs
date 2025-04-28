#![cfg_attr(docsrs, doc(cfg(feature = "json-serde")))]
#![cfg(feature = "json-serde")]

//! Defines a [`FileFormat`] using the JSON data format.

pub extern crate serde_json as original;

use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use singlefile::{FileFormat, FileFormatUtf8};

use std::io::{Read, Write};

/// An error that can occur while using [`Json`].
pub type JsonError = serde_json::Error;

/// A [`FileFormat`] corresponding to the JSON data format.
/// Implemented using the [`serde_json`] crate, only compatible with [`serde`] types.
///
/// This type provides an optional constant generic parameter for configuring pretty-print.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Json<const PRETTY: bool = true>;

impl<T, const PRETTY: bool> FileFormat<T> for Json<PRETTY>
where T: Serialize + DeserializeOwned {
  type FormatError = JsonError;

  fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    serde_json::from_reader(reader)
  }

  fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
    match PRETTY {
      true => serde_json::to_writer_pretty(writer, value),
      false => serde_json::to_writer(writer, value)
    }
  }

  fn to_buffer(&self, value: &T) -> Result<Vec<u8>, Self::FormatError> {
    match PRETTY {
      true => serde_json::to_vec_pretty(value),
      false => serde_json::to_vec(value)
    }
  }
}

impl<T, const PRETTY: bool> FileFormatUtf8<T> for Json<PRETTY>
where T: Serialize + DeserializeOwned {
  fn from_string_buffer(&self, buf: &str) -> Result<T, Self::FormatError> {
    serde_json::from_str(buf)
  }

  fn to_string_buffer(&self, value: &T) -> Result<String, Self::FormatError> {
    match PRETTY {
      true => serde_json::to_string_pretty(value),
      false => serde_json::to_string(value)
    }
  }
}

/// A shortcut type to a [`Json`] with pretty-print enabled.
pub type PrettyJson = Json<true>;
/// A shortcut type to a [`Json`] with pretty-print disabled.
pub type RegularJson = Json<false>;

/// A shortcut type to a [`Compressed`][crate::compression::Compressed] [`Json`].
/// Provides parameters for compression format and pretty-print configuration (defaulting to off).
#[cfg_attr(docsrs, doc(cfg(feature = "compression")))]
#[cfg(feature = "compression")]
pub type CompressedJson<C, const PRETTY: bool = false> = crate::compression::Compressed<C, Json<PRETTY>>;

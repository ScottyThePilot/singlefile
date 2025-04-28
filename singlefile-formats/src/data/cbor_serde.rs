#![cfg_attr(docsrs, doc(cfg(feature = "cbor-serde")))]
#![cfg(feature = "cbor-serde")]

//! Defines a [`FileFormat`] using the CBOR binary data format.

pub extern crate ciborium as original;

use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use singlefile::FileFormat;
use thiserror::Error;

use std::io::{Read, Write};

/// An error that can occur while using [`Cbor`].
#[derive(Debug, Error)]
pub enum CborError {
  /// An error occurred while serializing.
  #[error(transparent)]
  SerializeError(#[from] ciborium::ser::Error<std::io::Error>),
  /// An error occurred while deserializing.
  #[error(transparent)]
  DeserializeError(#[from] ciborium::de::Error<std::io::Error>)
}

/// A [`FileFormat`] corresponding to the CBOR binary data format.
/// Implemented using the [`ciborium`] crate, only compatible with [`serde`] types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cbor;

impl<T> FileFormat<T> for Cbor
where T: Serialize + DeserializeOwned {
  type FormatError = CborError;

  fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    ciborium::de::from_reader(reader).map_err(From::from)
  }

  fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
    ciborium::ser::into_writer(value, writer).map_err(From::from)
  }
}

/// A shortcut type to a [`Compressed`][crate::compression::Compressed] [`Cbor`].
/// Provides a single parameter for compression format.
#[cfg_attr(docsrs, doc(cfg(feature = "compression")))]
#[cfg(feature = "compression")]
pub type CompressedCbor<C> = crate::compression::Compressed<C, Cbor>;

#![cfg_attr(docsrs, doc(cfg(feature = "bincode")))]
#![cfg(feature = "bincode")]

//! Defines a [`FileFormat`] using the Bincode binary data format.

pub extern crate bincode as original;

use bincode::{Decode, Encode};
use bincode::config::{Configuration, Config, BigEndian, LittleEndian, Varint, Fixint, Limit, NoLimit};
use bincode::error::{DecodeError, EncodeError};
#[cfg(feature = "bincode-serde")]
use serde::ser::Serialize;
#[cfg(feature = "bincode-serde")]
use serde::de::DeserializeOwned;
use singlefile::FileFormat;
use thiserror::Error;

use std::io::{Read, Write};

/// An error that can occur while using [`Bincode`].
#[derive(Debug, Error)]
pub enum BincodeError {
  /// An error occurred while encoding.
  #[error(transparent)]
  EncodeError(#[from] EncodeError),
  /// An error occurred while decoding.
  #[error(transparent)]
  DecodeError(#[from] DecodeError)
}

/// A [`FileFormat`] corresponding to the CBOR binary data format.
/// Implemented using the [`ciborium`] crate, only compatible with types that implement
/// [`Decode`] and [`Encode`].
#[derive(Debug, Clone, Copy, Default)]
pub struct Bincode<E = LittleEndian, I = Varint, L = NoLimit> {
  /// The internal [`Configuration`].
  pub configuration: Configuration<E, I, L>
}

impl<T, E, I, L> FileFormat<T> for Bincode<E, I, L>
where T: Decode<()> + Encode, Configuration<E, I, L>: Config {
  type FormatError = BincodeError;

  #[inline]
  fn from_reader<R: Read>(&self, mut reader: R) -> Result<T, Self::FormatError> {
    Ok(bincode::decode_from_std_read(&mut reader, self.configuration)?)
  }

  #[inline]
  fn from_buffer(&self, buf: &[u8]) -> Result<T, Self::FormatError> {
    Ok(bincode::decode_from_slice(buf, self.configuration)?.0)
  }

  #[inline]
  fn to_writer<W: Write>(&self, mut writer: W, value: &T) -> Result<(), Self::FormatError> {
    bincode::encode_into_std_write(value, &mut writer, self.configuration)?;
    Ok(())
  }

  #[inline]
  fn to_buffer(&self, value: &T) -> Result<Vec<u8>, Self::FormatError> {
    Ok(bincode::encode_to_vec(value, self.configuration)?)
  }
}

impl<E, I, L> Bincode<E, I, L> {
  /// Creates a new [`Bincode`] given a [`Configuration`].
  #[inline]
  pub const fn new(configuration: Configuration<E, I, L>) -> Self {
    Bincode { configuration }
  }

  /// Makes bincode encode all integer types in big endian.
  ///
  /// Applies [`with_big_endian`][Configuration::with_big_endian] to the wrapped [`Configuration`].
  #[inline]
  pub const fn with_big_endian(self) -> Bincode<BigEndian, I, L> {
    Bincode { configuration: self.configuration.with_big_endian() }
  }

  /// Makes bincode encode all integer types in little endian.
  ///
  /// Applies [`with_little_endian`][Configuration::with_little_endian] to the wrapped [`Configuration`].
  #[inline]
  pub const fn with_little_endian(self) -> Bincode<LittleEndian, I, L> {
    Bincode { configuration: self.configuration.with_little_endian() }
  }

  /// Makes bincode encode all integer types with a variable integer encoding.
  ///
  /// Applies [`with_variable_int_encoding`][Configuration::with_variable_int_encoding] to the wrapped [`Configuration`].
  #[inline]
  pub const fn with_variable_int_encoding(self) -> Bincode<E, Varint, L> {
    Bincode { configuration: self.configuration.with_variable_int_encoding() }
  }

  /// Fixed-size integer encoding.
  ///
  /// Applies [`with_fixed_int_encoding`][Configuration::with_fixed_int_encoding] to the wrapped [`Configuration`].
  #[inline]
  pub const fn with_fixed_int_encoding(self) -> Bincode<E, Fixint, L> {
    Bincode { configuration: self.configuration.with_fixed_int_encoding() }
  }

  /// Sets the byte limit to `limit`.
  ///
  /// Applies [`with_limit`][Configuration::with_limit] to the wrapped [`Configuration`].
  #[inline]
  pub const fn with_limit<const N: usize>(self) -> Bincode<E, I, Limit<N>> {
    Bincode { configuration: self.configuration.with_limit() }
  }

  /// Clear the byte limit.
  ///
  /// Applies [`with_no_limit`][Configuration::with_no_limit] to the wrapped [`Configuration`].
  #[inline]
  pub const fn with_no_limit(self) -> Bincode<E, I, NoLimit> {
    Bincode { configuration: self.configuration.with_no_limit() }
  }
}

/// A [`FileFormat`] corresponding to the CBOR binary data format.
/// Implemented using the [`ciborium`] crate, only compatible with types that implement
/// [`Decode`] and [`Encode`].
#[cfg(feature = "bincode-serde")]
#[derive(Debug, Clone, Copy, Default)]
pub struct BincodeSerde<E = LittleEndian, I = Varint, L = NoLimit> {
  /// The internal [`Configuration`].
  pub configuration: Configuration<E, I, L>
}

#[cfg(feature = "bincode-serde")]
impl<T, E, I, L> FileFormat<T> for BincodeSerde<E, I, L>
where T: DeserializeOwned + Serialize, Configuration<E, I, L>: Config {
  type FormatError = BincodeError;

  #[inline]
  fn from_reader<R: Read>(&self, mut reader: R) -> Result<T, Self::FormatError> {
    Ok(bincode::serde::decode_from_std_read(&mut reader, self.configuration)?)
  }

  #[inline]
  fn from_buffer(&self, buf: &[u8]) -> Result<T, Self::FormatError> {
    Ok(bincode::serde::decode_from_slice(buf, self.configuration)?.0)
  }

  #[inline]
  fn to_writer<W: Write>(&self, mut writer: W, value: &T) -> Result<(), Self::FormatError> {
    bincode::serde::encode_into_std_write(value, &mut writer, self.configuration)?;
    Ok(())
  }

  #[inline]
  fn to_buffer(&self, value: &T) -> Result<Vec<u8>, Self::FormatError> {
    Ok(bincode::serde::encode_to_vec(value, self.configuration)?)
  }
}

#[cfg(feature = "bincode-serde")]
impl<E, I, L> BincodeSerde<E, I, L> {
  /// Creates a new [`Bincode`] given a [`Configuration`].
  #[inline]
  pub const fn new(configuration: Configuration<E, I, L>) -> Self {
    BincodeSerde { configuration }
  }

  /// Makes bincode encode all integer types in big endian.
  ///
  /// Applies [`with_big_endian`][Configuration::with_big_endian] to the wrapped [`Configuration`].
  #[inline]
  pub const fn with_big_endian(self) -> BincodeSerde<BigEndian, I, L> {
    BincodeSerde { configuration: self.configuration.with_big_endian() }
  }

  /// Makes bincode encode all integer types in little endian.
  ///
  /// Applies [`with_little_endian`][Configuration::with_little_endian] to the wrapped [`Configuration`].
  #[inline]
  pub const fn with_little_endian(self) -> BincodeSerde<LittleEndian, I, L> {
    BincodeSerde { configuration: self.configuration.with_little_endian() }
  }

  /// Makes bincode encode all integer types with a variable integer encoding.
  ///
  /// Applies [`with_variable_int_encoding`][Configuration::with_variable_int_encoding] to the wrapped [`Configuration`].
  #[inline]
  pub const fn with_variable_int_encoding(self) -> BincodeSerde<E, Varint, L> {
    BincodeSerde { configuration: self.configuration.with_variable_int_encoding() }
  }

  /// Fixed-size integer encoding.
  ///
  /// Applies [`with_fixed_int_encoding`][Configuration::with_fixed_int_encoding] to the wrapped [`Configuration`].
  #[inline]
  pub const fn with_fixed_int_encoding(self) -> BincodeSerde<E, Fixint, L> {
    BincodeSerde { configuration: self.configuration.with_fixed_int_encoding() }
  }

  /// Sets the byte limit to `limit`.
  ///
  /// Applies [`with_limit`][Configuration::with_limit] to the wrapped [`Configuration`].
  #[inline]
  pub const fn with_limit<const N: usize>(self) -> BincodeSerde<E, I, Limit<N>> {
    BincodeSerde { configuration: self.configuration.with_limit() }
  }

  /// Clear the byte limit.
  ///
  /// Applies [`with_no_limit`][Configuration::with_no_limit] to the wrapped [`Configuration`].
  #[inline]
  pub const fn with_no_limit(self) -> BincodeSerde<E, I, NoLimit> {
    BincodeSerde { configuration: self.configuration.with_no_limit() }
  }
}

/// A shortcut type to a [`Compressed`][crate::compression::Compressed] [`Bincode`].
/// Provides a single parameter for compression format.
#[cfg_attr(docsrs, doc(cfg(feature = "compression")))]
#[cfg(feature = "compression")]
pub type CompressedBincode<C, E = LittleEndian, I = Varint, L = NoLimit>
  = crate::compression::Compressed<C, Bincode<E, I, L>>;

/// A shortcut type to a [`Compressed`][crate::compression::Compressed] [`BincodeSerde`].
/// Provides a single parameter for compression format.
#[cfg_attr(docsrs, doc(cfg(all(feature = "compression", feature = "bincode-serde"))))]
#[cfg(all(feature = "compression", feature = "bincode-serde"))]
pub type CompressedBincodeSerde<C, E = LittleEndian, I = Varint, L = NoLimit>
  = crate::compression::Compressed<C, Bincode<E, I, L>>;

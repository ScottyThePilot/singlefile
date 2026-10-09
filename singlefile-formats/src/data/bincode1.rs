#![cfg_attr(docsrs, doc(cfg(feature = "bincode1")))]
#![cfg(feature = "bincode1")]

//! Defines a [`FileFormat`] using the Bincode binary data format.

pub extern crate bincode1 as original;

use bincode1::config::{
  Options, DefaultOptions,
  BigEndian, LittleEndian, NativeEndian,
  FixintEncoding, VarintEncoding,
  Bounded, Infinite,
  AllowTrailing, RejectTrailing,
  WithOtherEndian,
  WithOtherIntEncoding,
  WithOtherLimit,
  WithOtherTrailing
};
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use singlefile::FileFormat;

use std::io::{Read, Write};

/// An error that can occur while using [`Bincode`].
pub type BincodeError = bincode1::Error;

/// A [`FileFormat`] corresponding to the CBOR binary data format.
/// Implemented using the [`bincode`][bincode2] crate, only compatible with [`serde`] types.
#[derive(Debug, Clone, Copy, Default)]
pub struct Bincode<O: Options + Clone = DefaultOptions> {
  /// The internal [`Options`].
  pub options: O
}

impl<T, O> FileFormat<T> for Bincode<O>
where T: DeserializeOwned + Serialize, O: Options + Clone {
  type FormatError = BincodeError;

  #[inline]
  fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    self.options.clone().deserialize_from(reader)
  }

  #[inline]
  fn from_buffer(&self, buf: &[u8]) -> Result<T, Self::FormatError> {
    self.options.clone().deserialize(buf)
  }

  #[inline]
  fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
    self.options.clone().serialize_into(writer, value)
  }

  #[inline]
  fn to_buffer(&self, value: &T) -> Result<Vec<u8>, Self::FormatError> {
    self.options.clone().serialize(value)
  }
}

impl Bincode {
  /// Creates a new [`Bincode`] using [`DefaultOptions`].
  #[inline]
  pub fn new() -> Self {
    Bincode { options: DefaultOptions::new() }
  }
}

impl<O: Options + Clone> Bincode<O> {
  /// Sets the endianness to little-endian.
  ///
  /// Applies [`with_little_endian`][Options::with_little_endian] to the wrapped [`Options`].
  #[inline]
  pub fn with_little_endian(self) -> Bincode<WithOtherEndian<O, LittleEndian>> {
    Bincode { options: self.options.with_little_endian() }
  }

  /// Sets the endianness to big-endian.
  ///
  /// Applies [`with_big_endian`][Options::with_big_endian] to the wrapped [`Options`].
  #[inline]
  pub fn with_big_endian(self) -> Bincode<WithOtherEndian<O, BigEndian>> {
    Bincode { options: self.options.with_big_endian() }
  }

  /// Sets the endianness to the the machine-native endianness.
  ///
  /// Applies [`with_native_endian`][Options::with_native_endian] to the wrapped [`Options`].
  #[inline]
  pub fn with_native_endian(self) -> Bincode<WithOtherEndian<O, NativeEndian>> {
    Bincode { options: self.options.with_native_endian() }
  }

  /// Sets the length encoding to varint.
  ///
  /// Applies [`with_varint_encoding`][Options::with_varint_encoding] to the wrapped [`Options`].
  #[inline]
  pub fn with_varint_encoding(self) -> Bincode<WithOtherIntEncoding<O, VarintEncoding>> {
    Bincode { options: self.options.with_varint_encoding() }
  }

  /// Sets the length encoding to be fixed.
  ///
  /// Applies [`with_fixint_encoding`][Options::with_fixint_encoding] to the wrapped [`Options`].
  #[inline]
  pub fn with_fixint_encoding(self) -> Bincode<WithOtherIntEncoding<O, FixintEncoding>> {
    Bincode { options: self.options.with_fixint_encoding() }
  }

  /// Sets the byte limit to be unlimited.
  ///
  /// Applies [`with_no_limit`][Options::with_no_limit] to the wrapped [`Options`].
  #[inline]
  pub fn with_no_limit(self) -> Bincode<WithOtherLimit<O, Infinite>> {
    Bincode { options: self.options.with_no_limit() }
  }

  /// Sets the byte limit to `limit`.
  ///
  /// Applies [`with_limit`][Options::with_limit] to the wrapped [`Options`].
  #[inline]
  pub fn with_limit(self, limit: u64) -> Bincode<WithOtherLimit<O, Bounded>> {
    Bincode { options: self.options.with_limit(limit) }
  }

  /// Sets the deserializer to reject trailing bytes.
  ///
  /// Applies [`reject_trailing_bytes`][Options::reject_trailing_bytes] to the wrapped [`Options`].
  #[inline]
  pub fn reject_trailing_bytes(self) -> Bincode<WithOtherTrailing<O, RejectTrailing>> {
    Bincode { options: self.options.reject_trailing_bytes() }
  }

  /// Sets the deserializer to allow trailing bytes.
  ///
  /// Applies [`allow_trailing_bytes`][Options::allow_trailing_bytes] to the wrapped [`Options`].
  #[inline]
  pub fn allow_trailing_bytes(self) -> Bincode<WithOtherTrailing<O, AllowTrailing>> {
    Bincode { options: self.options.allow_trailing_bytes() }
  }
}

/// A shortcut type to a [`Compressed`][crate::compression::Compressed] [`Bincode`].
/// Provides a single parameter for compression format.
#[cfg_attr(docsrs, doc(cfg(feature = "compression")))]
#[cfg(feature = "compression")]
pub type CompressedBincode<C, O = DefaultOptions>
  = crate::compression::Compressed<C, Bincode<O>>;

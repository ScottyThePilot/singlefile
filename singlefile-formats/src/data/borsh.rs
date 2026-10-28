#![cfg_attr(docsrs, doc(cfg(feature = "borsh")))]
#![cfg(feature = "borsh")]

//! Defines a [`FileFormat`] using the Borsh binary data format.

pub extern crate borsh as original;

use borsh::de::BorshDeserialize;
use borsh::ser::BorshSerialize;
use singlefile::FileFormat;

use std::io::{Read, Write};

/// An error that can occur while using [`Borsh`].
pub type BorshError = borsh::io::Error;

/// A [`FileFormat`] corresponding to the Borsh binary data format.
/// Implemented using the [`borsh`] crate, only compatible with types
/// that implement [`BorshDeserialize`] and [`BorshSerialize`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Borsh;

impl<T> FileFormat<T> for Borsh
where T: BorshDeserialize + BorshSerialize {
  type FormatError = BorshError;

  fn from_reader<R: Read>(&self, mut reader: R) -> Result<T, Self::FormatError> {
    BorshDeserialize::try_from_reader(&mut reader)
  }

  fn from_buffer(&self, buf: &[u8]) -> Result<T, Self::FormatError> {
    BorshDeserialize::try_from_slice(buf)
  }

  fn to_writer<W: Write>(&self, mut writer: W, value: &T) -> Result<(), Self::FormatError> {
    BorshSerialize::serialize(value, &mut writer)
  }
}

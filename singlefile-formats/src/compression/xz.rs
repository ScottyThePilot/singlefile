#![cfg_attr(docsrs, doc(cfg(feature = "xz")))]
#![cfg(feature = "xz")]

//! Defines a [`CompressionFormat`] for the LZMA/XZ compression algorithm.

pub extern crate xz2 as original;

use crate::compression::{CompressionFormat, CompressionFormatLevels};

use std::io::{Read, Write};

/// A [`CompressionFormat`] corresponding to the LZMA/XZ compression algorithm.
/// Implemented using the [`xz2`] crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Xz;

impl CompressionFormat for Xz {
  type Encoder<W: Write> = xz2::write::XzEncoder::<W>;
  type Decoder<R: Read> = xz2::read::XzDecoder::<R>;

  fn encode_writer<W: Write>(&self, writer: W, compression: u32) -> Self::Encoder<W> {
    Self::Encoder::new(writer, compression)
  }

  fn decode_reader<R: Read>(&self, reader: R) -> Self::Decoder<R> {
    Self::Decoder::new(reader)
  }
}

impl CompressionFormatLevels for Xz {
  const COMPRESSION_LEVEL_NONE: u32 = 0;
  const COMPRESSION_LEVEL_FAST: u32 = 1;
  const COMPRESSION_LEVEL_BEST: u32 = 9;
  const COMPRESSION_LEVEL_DEFAULT: u32 = 6;
}

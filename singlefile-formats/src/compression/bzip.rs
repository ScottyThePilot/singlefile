#![cfg_attr(docsrs, doc(cfg(feature = "bzip")))]
#![cfg(feature = "bzip")]

//! Defines a [`CompressionFormat`] for the bzip compression algorithm.

pub extern crate bzip2 as original;

use crate::compression::{CompressionFormat, CompressionFormatLevels};

use std::io::{Read, Write};

/// A [`CompressionFormat`] corresponding to the bzip compression algorithm.
/// Implemented using the [`bzip2`] crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BZip2;

impl CompressionFormat for BZip2 {
  type Encoder<W: Write> = bzip2::write::BzEncoder::<W>;
  type Decoder<R: Read> = bzip2::read::BzDecoder::<R>;

  fn encode_writer<W: Write>(&self, writer: W, level: u32) -> Self::Encoder<W> {
    Self::Encoder::new(writer, bzip2::Compression::new(level))
  }

  fn decode_reader<R: Read>(&self, reader: R) -> Self::Decoder<R> {
    Self::Decoder::new(reader)
  }
}

impl CompressionFormatLevels for BZip2 {
  const COMPRESSION_LEVEL_NONE: u32 = 0;
  const COMPRESSION_LEVEL_FAST: u32 = 1;
  const COMPRESSION_LEVEL_BEST: u32 = 9;
  const COMPRESSION_LEVEL_DEFAULT: u32 = 6;
}

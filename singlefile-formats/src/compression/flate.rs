#![cfg_attr(docsrs, doc(cfg(feature = "flate")))]
#![cfg(feature = "flate")]

//! Defines [`CompressionFormat`]s for the DEFLATE, gzip and zlib compression algorithms.

pub extern crate flate2 as original;

use crate::compression::{CompressionFormat, CompressionFormatLevels};

use std::io::{Read, Write};

/// A [`CompressionFormat`] corresponding to the DEFLATE compression algorithm.
/// Implemented using the [`flate2`] crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Deflate;

impl CompressionFormat for Deflate {
  type Encoder<W: Write> = flate2::write::DeflateEncoder::<W>;
  type Decoder<R: Read> = flate2::read::DeflateDecoder::<R>;

  fn encode_writer<W: Write>(&self, writer: W, compression: u32) -> Self::Encoder<W> {
    Self::Encoder::new(writer, flate2::Compression::new(compression))
  }

  fn decode_reader<R: Read>(&self, reader: R) -> Self::Decoder<R> {
    Self::Decoder::new(reader)
  }
}

impl CompressionFormatLevels for Deflate {
  const COMPRESSION_LEVEL_NONE: u32 = 0;
  const COMPRESSION_LEVEL_FAST: u32 = 1;
  const COMPRESSION_LEVEL_BEST: u32 = 9;
  const COMPRESSION_LEVEL_DEFAULT: u32 = 6;
}

/// A [`CompressionFormat`] corresponding to the gzip compression algorithm.
/// Implemented using the [`flate2`] crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Gz;

impl CompressionFormat for Gz {
  type Encoder<W: Write> = flate2::write::GzEncoder::<W>;
  type Decoder<R: Read> = flate2::read::GzDecoder::<R>;

  fn encode_writer<W: Write>(&self, writer: W, compression: u32) -> Self::Encoder<W> {
    Self::Encoder::new(writer, flate2::Compression::new(compression))
  }

  fn decode_reader<R: Read>(&self, reader: R) -> Self::Decoder<R> {
    Self::Decoder::new(reader)
  }
}

impl CompressionFormatLevels for Gz {
  const COMPRESSION_LEVEL_NONE: u32 = 0;
  const COMPRESSION_LEVEL_FAST: u32 = 1;
  const COMPRESSION_LEVEL_BEST: u32 = 9;
  const COMPRESSION_LEVEL_DEFAULT: u32 = 6;
}

/// A [`CompressionFormat`] corresponding to the zlib compression algorithm.
/// Implemented using the [`flate2`] crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ZLib;

impl CompressionFormat for ZLib {
  type Encoder<W: Write> = flate2::write::ZlibEncoder::<W>;
  type Decoder<R: Read> = flate2::read::ZlibDecoder::<R>;

  fn encode_writer<W: Write>(&self, writer: W, compression: u32) -> Self::Encoder<W> {
    Self::Encoder::new(writer, flate2::Compression::new(compression))
  }

  fn decode_reader<R: Read>(&self, reader: R) -> Self::Decoder<R> {
    Self::Decoder::new(reader)
  }
}

impl CompressionFormatLevels for ZLib {
  const COMPRESSION_LEVEL_NONE: u32 = 0;
  const COMPRESSION_LEVEL_FAST: u32 = 1;
  const COMPRESSION_LEVEL_BEST: u32 = 9;
  const COMPRESSION_LEVEL_DEFAULT: u32 = 6;
}

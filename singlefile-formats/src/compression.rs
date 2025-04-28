#![cfg_attr(docsrs, doc(cfg(feature = "compression")))]
#![cfg(feature = "compression")]

//! Defines a compression format interface, and a [`FileFormat`] which wraps another [`FileFormat`],
//! is generic over compression formats, and compresses the contents of the wrapped format.

pub mod bzip;
pub mod flate;
pub mod xz;

use singlefile::FileFormat;

use std::io::{Read, Write};

/// Combines a [`FileFormat`] and a [`CompressionFormat`], making the contents emitted by
/// the format compressed before writing to disk, and decompressed before parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Compressed<C, F> {
  /// The [`FileFormat`] to be used.
  pub format: F,
  /// The [`CompressionFormat`] to be used.
  pub compression: C,
  /// The level of compression to use.
  /// This value may have different meanings for different compression formats.
  pub level: u32
}

impl<C, F> Compressed<C, F> {
  /// Create a new [`Compressed`], given a compression level.
  #[inline]
  pub const fn with_level(format: F, compression: C, level: u32) -> Self {
    Compressed { format, compression, level }
  }
}

impl<C, F> Compressed<C, F> where C: CompressionFormatLevels {
  /// Creates a new [`Compressed`] with the default compression level.
  #[inline]
  pub const fn new(format: F, compression: C) -> Self {
    Compressed::with_level(format, compression, C::COMPRESSION_LEVEL_DEFAULT)
  }

  /// Creates a new [`Compressed`] with the 'fast' compression level.
  #[inline]
  pub const fn new_fast_compression(format: F, compression: C) -> Self {
    Compressed::with_level(format, compression, C::COMPRESSION_LEVEL_FAST)
  }

  /// Creates a new [`Compressed`] with the 'best' compression level.
  #[inline]
  pub const fn new_best_compression(format: F, compression: C) -> Self {
    Compressed::with_level(format, compression, C::COMPRESSION_LEVEL_BEST)
  }
}

impl<C, F> Default for Compressed<C, F>
where C: Default + CompressionFormatLevels, F: Default {
  #[inline]
  fn default() -> Self {
    Compressed::new(F::default(), C::default())
  }
}

impl<T, C, F> FileFormat<T> for Compressed<C, F>
where C: CompressionFormat, F: FileFormat<T> {
  type FormatError = F::FormatError;

  fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    self.format.from_reader(self.compression.decode_reader(reader))
  }

  fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
    self.format.to_writer(self.compression.encode_writer(writer, self.level), value)
  }
}

/// Defines a format for lossless compression of arbitrary data.
///
/// In order to use a [`CompressionFormat`], you may consider using the [`Compressed`] struct.
pub trait CompressionFormat {
  /// The encoder wrapper type that compresses data sent to the contained writer.
  type Encoder<W: Write>: Write;
  /// The decoder wrapper type that decompresses data sent from the contained reader.
  type Decoder<R: Read>: Read;

  /// Wraps a writer that takes uncompressed data, producing a new writer that outputs compressed data.
  fn encode_writer<W: Write>(&self, writer: W, level: u32) -> Self::Encoder<W>;
  /// Wraps a reader that takes compressed data, producing a new reader that outputs uncompressed data.
  fn decode_reader<R: Read>(&self, reader: R) -> Self::Decoder<R>;
}

/// Defines compression level presets for a [`CompressionFormat`].
pub trait CompressionFormatLevels: CompressionFormat {
  /// The level for no compression.
  const COMPRESSION_LEVEL_NONE: u32;
  /// The level for 'fast' compression.
  const COMPRESSION_LEVEL_FAST: u32;
  /// The level for 'best' compression.
  const COMPRESSION_LEVEL_BEST: u32;
  /// The level for default compression.
  const COMPRESSION_LEVEL_DEFAULT: u32;
}

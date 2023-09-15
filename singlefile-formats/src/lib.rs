//! This library provides a number of default implementations
//! of [`FileFormat`]s for use within [`singlefile`].

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(
  future_incompatible,
  missing_copy_implementations,
  missing_debug_implementations,
  missing_docs,
  unreachable_pub
)]

pub extern crate singlefile;

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

/// Defines a [`FileFormat`] using the CBOR binary data format.
#[cfg_attr(docsrs, doc(cfg(feature = "cbor-serde")))]
#[cfg(feature = "cbor-serde")]
pub mod cbor_serde {
  pub extern crate ciborium;

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

  /// A shortcut type to a [`Compressed`][crate::Compressed] [`Cbor`].
  /// Provides a single parameter for compression format.
  pub type CompressedCbor<C> = crate::Compressed<C, Cbor>;
}

/// Defines a [`FileFormat`] using the JSON data format.
#[cfg_attr(docsrs, doc(cfg(feature = "json-serde")))]
#[cfg(feature = "json-serde")]
pub mod json_serde {
  pub extern crate serde_json;

  use serde::ser::Serialize;
  use serde::de::DeserializeOwned;
  use singlefile::FileFormat;

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

  /// A shortcut type to a [`Json`] with pretty-print enabled.
  pub type PrettyJson = Json<true>;
  /// A shortcut type to a [`Json`] with pretty-print disabled.
  pub type RegularJson = Json<false>;

  /// A shortcut type to a [`Compressed`][crate::Compressed] [`Json`].
  /// Provides parameters for compression format and pretty-print configuration (defaulting to off).
  pub type CompressedJson<C, const PRETTY: bool = false> = crate::Compressed<C, Json<PRETTY>>;
}

/// Defines a [`FileFormat`] using the TOML data format.
#[cfg_attr(docsrs, doc(cfg(feature = "toml-serde")))]
#[cfg(feature = "toml-serde")]
pub mod toml_serde {
  pub extern crate toml;

  use serde::ser::Serialize;
  use serde::de::DeserializeOwned;
  use singlefile::FileFormat;
  use thiserror::Error;

  use std::io::{Read, Write};

  /// An error that can occur while using [`Toml`].
  #[derive(Debug, Error)]
  pub enum TomlError {
    /// An error occured while reading data to the string buffer.
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    /// An error occurred while serializing.
    #[error(transparent)]
    SerializeError(#[from] toml::ser::Error),
    /// An error occurred while deserializing.
    #[error(transparent)]
    DeserializeError(#[from] toml::de::Error)
  }

  /// A [`FileFormat`] corresponding to the TOML data format.
  /// Implemented using the [`toml`] crate, only compatible with [`serde`] types.
  ///
  /// This type provides an optional constant generic parameter for configuring pretty-print.
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
  pub struct Toml<const PRETTY: bool = true>;

  impl<T, const PRETTY: bool> FileFormat<T> for Toml<PRETTY>
  where T: Serialize + DeserializeOwned {
    type FormatError = TomlError;

    fn from_reader<R: Read>(&self, mut reader: R) -> Result<T, Self::FormatError> {
      let mut buf = String::new();
      reader.read_to_string(&mut buf)?;
      toml::de::from_str(&buf).map_err(From::from)
    }

    #[inline]
    fn from_reader_buffered<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
      // no need to pass `reader` in with a `BufReader` as that would cause things to be buffered twice
      self.from_reader(reader)
    }

    fn to_writer<W: Write>(&self, mut writer: W, value: &T) -> Result<(), Self::FormatError> {
      let buf = self.to_buffer(value)?;
      writer.write_all(&buf).map_err(From::from)
    }

    #[inline]
    fn to_writer_buffered<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
      // no need to pass `writer` in with a `BufWriter` as that would cause things to be buffered twice
      self.to_writer(writer, value)
    }

    fn to_buffer(&self, value: &T) -> Result<Vec<u8>, Self::FormatError> {
      Ok(match PRETTY {
        true => toml::ser::to_string_pretty(value),
        false => toml::ser::to_string(value)
      }?.into_bytes())
    }
  }

  /// A shortcut type to a [`Toml`] with pretty-print enabled.
  pub type PrettyToml = Toml<true>;
  /// A shortcut type to a [`Toml`] with pretty-print disabled.
  pub type RegularToml = Toml<false>;

  /// A shortcut type to a [`Compressed`][crate::Compressed] [`Toml`].
  /// Provides parameters for compression format and pretty-print configuration (defaulting to off).
  pub type CompressedToml<C, const PRETTY: bool = false> = crate::Compressed<C, Toml<PRETTY>>;
}

/// Defines a [`CompressionFormat`] for the bzip compression algorithm.
#[cfg_attr(docsrs, doc(cfg(feature = "bzip")))]
#[cfg(feature = "bzip")]
pub mod bzip {
  pub extern crate bzip2;

  use crate::{CompressionFormat, CompressionFormatLevels};

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
}

/// Defines [`CompressionFormat`]s for the DEFLATE, gzip and zlib compression algorithms.
#[cfg_attr(docsrs, doc(cfg(feature = "flate")))]
#[cfg(feature = "flate")]
pub mod flate {
  pub extern crate flate2;

  use crate::{CompressionFormat, CompressionFormatLevels};

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
}

/// Defines a [`CompressionFormat`] for the LZMA/XZ compression algorithm.
#[cfg_attr(docsrs, doc(cfg(feature = "xz")))]
#[cfg(feature = "xz")]
pub mod xz {
  pub extern crate xz2;

  use crate::{CompressionFormat, CompressionFormatLevels};

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
}

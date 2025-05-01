//! This library provides a number of default [`FileFormat`] implementations
//! for use within [`singlefile`](https://crates.io/crates/singlefile).
//!
//! # Features
//! By default, no features are enabled.
//!
//! - `bincode`: Enables the [`Bincode`] file format.
//! - `bincode-serde`: Enables the [`BincodeSerde`] file format for use with [`serde`] types.
//! - `cbor-serde`: Enables the [`Cbor`] file format for use with [`serde`] types.
//! - `json-serde`: Enables the [`Json`] file format for use with [`serde`] types.
//! - `toml-serde`: Enables the [`Toml`] file format for use with [`serde`] types.
//! - `bzip`: Enables the [`BZip2`] compression format. See [`CompressionFormat`] for more info.
//! - `bzip-rust`: Enables the `libbz2-rs-sys` feature for `bzip2`.
//! - `flate`: Enables the [`Deflate`], [`Gz`],
//!   and [`ZLib`] compression formats. See [`CompressionFormat`] for more info.
//! - `xz`: Enables the [`Xz`] compression format. See [`CompressionFormat`] for more info.
//!
//! [`FileFormat`]: singlefile::FileFormat
//! [`Bincode`]: crate::data::bincode::Bincode
//! [`BincodeSerde`]: crate::data::bincode::BincodeSerde
//! [`Cbor`]: crate::data::cbor_serde::Cbor
//! [`Json`]: crate::data::json_serde::Json
//! [`Toml`]: crate::data::toml_serde::Toml
//! [`CompressionFormat`]: crate::compression::CompressionFormat
//! [`BZip2`]: crate::compression::bzip::BZip2
//! [`Deflate`]: crate::compression::flate::Deflate
//! [`Gz`]: crate::compression::flate::Gz
//! [`ZLib`]: crate::compression::flate::ZLib
//! [`Xz`]: crate::compression::xz::Xz

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

pub mod compression;
pub mod data;
pub mod utils_serde;

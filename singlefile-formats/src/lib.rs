//! This library provides a number of default [`FileFormat`] implementations
//! for use within [`singlefile`](https://crates.io/crates/singlefile).
//!
//! ## Features
//! By default, only the `compression` feature is enabled.
//!
//! - `compression`: Enables the [`compression`] module. Enabled by default.
//! - `utils-serde`: Enables the [`utils_serde`] module.
//! - `base64`: Enables the [`Base64`][crate::data::base64::Base64] file format.
//! - `bincode1`: Enables the [`Bincode`][crate::data::bincode1::Bincode] file format.
//! - `bincode2`: Enables the [`Bincode`][crate::data::bincode2::Bincode] file format.
//! - `bincode2-serde`: Enables the [`BincodeSerde`][crate::data::bincode2::BincodeSerde] file format for use with [`serde`] types.
//! - `bincode-reloaded`: Enables the [`Bincode`][crate::data::bincode_reloaded::Bincode] file format.
//! - `bincode-reloaded-serde`: Enables the [`BincodeSerde`][crate::data::bincode_reloaded::BincodeSerde] file format for use with [`serde`] types.
//! - `cbor-serde`: Enables the [`Cbor`][crate::data::cbor_serde::Cbor] file format for use with [`serde`] types.
//! - `json-serde`: Enables the [`Json`][crate::data::json_serde::Json] file format for use with [`serde`] types.
//! - `ron-serde`: Enables the [`Ron`][crate::data::ron_serde::Ron] file format for use with [`serde`] types.
//! - `toml-serde`: Enables the [`Toml`][crate::data::toml_serde::Toml] file format for use with [`serde`] types.
//! - `bzip`: Enables the [`BZip2`][crate::compression::bzip::BZip2] compression format. See [`CompressionFormat`] for more info.
//! - `flate`: Enables the [`Deflate`][crate::compression::flate::Deflate], [`Gz`][crate::compression::flate::Gz], and
//!   [`ZLib`][crate::compression::flate::ZLib] compression formats. See [`CompressionFormat`] for more info.
//! - `xz`: Enables the [`Xz`][crate::compression::xz::Xz] compression format. See [`CompressionFormat`] for more info.
//!
//! [`FileFormat`]: singlefile::FileFormat
//! [`CompressionFormat`]: crate::compression::CompressionFormat

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(
  absolute_paths_not_starting_with_crate,
  redundant_imports,
  redundant_lifetimes,
  future_incompatible,
  deprecated_in_future,
  missing_copy_implementations,
  missing_debug_implementations,
  missing_docs,
  unreachable_pub
)]

pub extern crate singlefile;

pub mod compression;
pub mod data;
pub mod utils_serde;

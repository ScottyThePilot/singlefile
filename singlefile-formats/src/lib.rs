//! This library provides a number of default [`FileFormat`] implementations
//! for use within [`singlefile`](https://crates.io/crates/singlefile).
//!
//! # Features
//! By default, no features are enabled.
//!
//! - `cbor-serde`: Enables the [`Cbor`][crate::cbor_serde::Cbor] file format for use with [`serde`] types.
//! - `json-serde`: Enables the [`Json`][crate::json_serde::Json] file format for use with [`serde`] types.
//! - `toml-serde`: Enables the [`Toml`][crate::toml_serde::Toml] file format for use with [`serde`] types.
//! - `bzip`: Enables the [`BZip2`][crate::bzip::BZip2] compression format. See [`CompressionFormat`] for more info.
//! - `flate`: Enables the [`Deflate`][crate::flate::Deflate], [`Gz`][crate::flate::Gz],
//!   and [`ZLib`][crate::flate::ZLib] compression formats. See [`CompressionFormat`] for more info.
//! - `xz`: Enables the [`Xz`][crate::xz::Xz] compression format. See [`CompressionFormat`] for more info.

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

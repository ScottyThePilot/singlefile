//! This library is designed to be a dead-simple way of reading and writing your rust values to and from disk.
//!
//! ## Usage
//! `singlefile` provides a generic [`Container`] type, along with type alias variants for different use cases.
//! [`Container`] is named so to indicate that it contains and manages a file and a value.
//!
//! ```no_run
//! # use singlefile_formats::data::json_serde::{Json, JsonError};
//! // A readable, writable container
//! use singlefile::container::ContainerWritable;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, Default)]
//! struct MyData {
//!   magic_number: i32
//! }
//!
//! // Attempts to open 'my_data.json', creating it from default if it does not exist,
//! // expecting data that the `Json` format can decode into `MyData`
//! let mut my_container = ContainerWritable::<MyData, Json>::create_or_default("my_data.json", Json)?;
//! // For regular `Container`s, `Deref` and `DerefMut` can be used to access the contained type
//! println!("magic_number: {}", my_container.magic_number); // 0 (as long as the file didn't exist before)
//! my_container.magic_number += 1;
//! // Write the new state of `MyData` to disk
//! my_container.commit()?;
//! # Ok::<(), singlefile::Error<JsonError>>(())
//! ```
//!
//! We'd then expect the resulting `my_data.json` to look like:
//!
//! ```json
//! {
//!   "magic_number": 1
//! }
//! ```
//!
//! ## Shared and async containers
//! `singlefile` also provides a [`ContainerShared`] type that can be used from multiple threads, as well as
//! a [`ContainerSharedAsync`] that can be used from multiple threads and spawns its operations asynchronously.
//! Currently, [`ContainerSharedAsync`] can only be guaranteed to work alongside Tokio.
//!
//! The shared container types can be enabled with the `shared` cargo feature.
//! The async container types can be enabled with the `shared-async` cargo feature.
//!
//! ```no_run
//! # use singlefile_formats::data::json_serde::{Json, JsonError};
//! # use std::convert::Infallible;
//! // A readable, writable container with multiple-ownership
//! use singlefile::container_shared::ContainerSharedWritable;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, Default)]
//! struct MyData {
//!   magic_number: i32
//! }
//!
//! // `ContainerShared` types may be cloned cheaply, they behave like `Arc`s
//! let my_container = ContainerSharedWritable::<MyData, Json>::create_or_default("my_data.json", Json)?;
//!
//! // Get access to the contained `MyData`, increment it, and commit changes to disk
//! std::thread::spawn(move || {
//!   my_container.operate_mut_commit(|my_data| {
//!     my_data.magic_number += 1;
//!     Ok::<(), Infallible>(())
//!   });
//! });
//! # Ok::<(), singlefile::Error<JsonError>>(())
//! ```
//!
//! ## File formats
//! `singlefile` is serialization framework-agnostic, so you will need a [`FileFormat`] adapter
//! before you are able to read and write a given file format to disk.
//!
//! Here is how you'd write a `Json` adapter for the above examples, using `serde`.
//!
//! ```no_run
//! # use singlefile_formats::data::json_serde::original as serde_json;
//! use serde::ser::Serialize;
//! use serde::de::DeserializeOwned;
//! use singlefile::FileFormat;
//! use std::io::{Read, Write};
//!
//! struct Json;
//!
//! impl<T> FileFormat<T> for Json
//! where T: Serialize + DeserializeOwned {
//!   type FormatError = serde_json::Error;
//!
//!   fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
//!     serde_json::to_writer_pretty(writer, value).map_err(From::from)
//!   }
//!
//!   fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
//!     serde_json::from_reader(reader).map_err(From::from)
//!   }
//! }
//! ```
//!
//! Alternatively, you can use one of the preset file formats provided by
//! [`singlefile-formats`](https://crates.io/crates/singlefile-formats).
//!
//! ## Features
//! By default, only the `tokio-parking-lot` feature is enabled.
//!
//! - `shared`: Enables [`ContainerShared`], pulling in `parking_lot`.
//! - `shared-async`: Enables [`ContainerSharedAsync`], pulling in `tokio` and (by default) `parking_lot`.
//! - `fs-err2`: Enables this crate to use `fs-err` v2 for everything filesystem-related.
//! - `fs-err3`: Enables this crate to use `fs-err` v3 for everything filesystem-related. Overrides `fs-err2` if present.
//! - `fs-err`: An alias for the `fs-err3` feature (and will correspond to the latest version of `fs-err` going forwards).
//! - `deadlock-detection`: Enables `parking_lot`'s `deadlock_detection` feature, if it is present.
//! - `tokio-parking-lot`: Enables `parking_lot` for use in `tokio`, if it is present. Enabled by default.
//!
//! [`Container`]: crate::container::Container
//! [`ContainerShared`]: crate::container_shared::ContainerShared
//! [`ContainerSharedAsync`]: crate::container_shared_async::ContainerSharedAsync
//! [`FileFormat`]: crate::manager::format::FileFormat

#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
  clippy::wrong_self_convention
)]
#![warn(
  future_incompatible,
  missing_copy_implementations,
  missing_debug_implementations,
  missing_docs,
  unreachable_pub
)]

extern crate fs4;
extern crate thiserror;
#[cfg(feature = "shared")]
extern crate parking_lot;
#[cfg(feature = "shared-async")]
extern crate tokio;

pub mod container;
#[cfg_attr(docsrs, doc(cfg(feature = "shared")))]
#[cfg(feature = "shared")]
pub mod container_shared;
#[cfg_attr(docsrs, doc(cfg(feature = "shared-async")))]
#[cfg(feature = "shared-async")]
pub mod container_shared_async;
pub mod error;
pub mod fs;
pub mod manager;
pub mod utils;

pub use crate::error::{Error, UserError};

#[doc(inline)]
pub use crate::manager::format::{FileFormat, FileFormatUtf8};

pub(crate) mod sealed {
  pub trait Sealed {}
}

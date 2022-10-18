//! This library is designed to be a dead-simple way of accessing and manipulating files,
//! treating those files as if they represent some Rust value.
//!
//! ## Usage
//! `singlefile` provides a generic [`Container`] type, along with type alias variants for different use cases.
//! [`Container`] is named so to indicate that it contains and manages a file and a value.
//!
//! ```no_run
//! # singlefile::define_file_format_serde!(Json, serde_json::Error, serde_json::to_writer, serde_json::from_reader);
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
//! // expecting data that the `Json` format can decode into `MyData`.
//! let mut my_container = ContainerWritable::<MyData, Json>::create_or_default("my_data.json", Json)?;
//! // For regular `Container`s, `Deref` and `DerefMut` can be used to access the contained type
//! println!("magic_number: {}", my_container.magic_number); // 0 (as long as the file didn't exit before)
//! my_container.magic_number += 1;
//! // Write the new state of `MyData` to disk
//! my_container.commit()?;
//! # Ok::<(), singlefile::Error<serde_json::Error>>(())
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
//! a [`ContainerAsync`] that can be used from multiple threads and spawns its operations asynchronously.
//! Currently, [`ContainerAsync`] can only be guaranteed to work alongside Tokio.
//!
//! ```no_run
//! # singlefile::define_file_format_serde!(Json, serde_json::Error, serde_json::to_writer, serde_json::from_reader);
//! # use std::convert::Infallible;
//! // A readable, writable container with multiple-ownership
//! use singlefile::container_shared::ContainerSharedWritable;
//! # use serde::{Serialize, Deserialize};
//! #
//! # #[derive(Serialize, Deserialize, Default)]
//! # struct MyData {
//! #   magic_number: i32
//! # }
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
//! # Ok::<(), singlefile::Error<serde_json::Error>>(())
//! ```
//!
//! ## File formats
//!
//! `singlefile` is serialization framework-agnostic, so you will need a [`FileFormat`] adapter
//! before you are able to read and write a given file format to disk.
//!
//! Here is how you'd write a `Json` adapter for the above examples, using `serde`.
//!
//! ```no_run
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
//! [`Container`]: crate::container::Container
//! [`ContainerShared`]: crate::container_shared::ContainerShared
//! [`ContainerAsync`]: crate::container_shared_async::ContainerAsync
//! [`FileFormat`]: crate::manager::format::FileFormat

#![warn(
  missing_copy_implementations,
  missing_debug_implementations,
  missing_docs,
  unreachable_pub
)]

extern crate fs4;
extern crate thiserror;
#[cfg(feature = "tokio")]
extern crate tokio;

pub mod container;
#[cfg(feature = "shared")]
pub mod container_shared;
#[cfg(feature = "shared-async")]
pub mod container_shared_async;
pub mod error;
pub mod manager;

pub use crate::container::Container;
pub use crate::container::ContainerReadonly;
pub use crate::container::ContainerWritable;
pub use crate::container::ContainerReadonlyLocked;
pub use crate::container::ContainerWritableLocked;

pub use crate::error::{Error, UserError};

#[doc(inline)]
pub use crate::manager::format::FileFormat;

#[doc(hidden)]
pub mod private {
  #[doc(hidden)]
  #[macro_export]
  macro_rules! define_file_format_serde {
    ($vis:vis $Type:ident, $Error:ty, $to_writer:expr, $from_reader:expr) => {
      #[derive(Debug, Clone, Copy)]
      $vis struct $Type;

      impl<T> $crate::FileFormat<T> for $Type
      where T: serde::ser::Serialize + serde::de::DeserializeOwned {
        type FormatError = $Error;

        fn to_writer<W: std::io::Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
          $to_writer(writer, value).map_err(From::from)
        }

        fn from_reader<R: std::io::Read>(&self, reader: R) -> Result<T, Self::FormatError> {
          $from_reader(reader).map_err(From::from)
        }
      }
    };
  }
}

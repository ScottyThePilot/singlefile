//! This crate exports a number of structs that allow you to do simple
//! operations on individual files that utilize serde-compatable formats.

#![warn(missing_debug_implementations)]
extern crate fs4;
extern crate serde;
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
pub use crate::container::ContainerMemoryOnly;
pub use crate::container::ContainerReadonly;
pub use crate::container::ContainerWritable;
pub use crate::container::ContainerReadonlyLocked;
pub use crate::container::ContainerWritableLocked;

pub use crate::error::{SingleFileError, SingleFileUserError};

pub use crate::manager::format::{FileFormat, FormatError};

#[doc(hidden)]
pub mod private {
  #[cfg(feature = "shared-async")]
  #[doc(hidden)]
  #[inline(always)]
  pub fn noop<T>(t: T) -> T { t }
}

#[doc(hidden)]
#[deprecated = "use `container_shared_async` instead"]
#[cfg(feature = "tokio")]
pub use crate::container_shared_async as container_tokio;

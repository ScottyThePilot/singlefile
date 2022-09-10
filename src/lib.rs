//! This crate exports a number of structs that allow you to do simple
//! operations on individual files that utilize serde-compatable formats.

#![warn(missing_debug_implementations)]
extern crate fs4;
extern crate serde;
#[cfg(feature = "tokio")]
extern crate tokio;

pub mod container_shared;
#[cfg(feature = "tokio")]
pub mod container_tokio;
pub mod container;
pub mod error;
pub mod manager;

pub use crate::container_shared::ContainerShared;
pub use crate::container_shared::ContainerSharedMutable;
pub use crate::container_shared::ContainerSharedReadonly;
pub use crate::container_shared::ContainerSharedWritable;
pub use crate::container_shared::ContainerSharedReadonlyLocked;
pub use crate::container_shared::ContainerSharedWritableLocked;

pub use crate::container::Container;
pub use crate::container::ContainerMemoryOnly;
pub use crate::container::ContainerReadonly;
pub use crate::container::ContainerWritable;
pub use crate::container::ContainerReadonlyLocked;
pub use crate::container::ContainerWritableLocked;

pub use crate::error::Error;

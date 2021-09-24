//! This crate exports a number of structs that allow you to do simple
//! operations on individual files that utilize serde-compatable formats.

#![warn(missing_debug_implementations)]
extern crate fs2;
extern crate serde;

pub mod backend_shared;
pub mod backend;
pub mod error;
pub mod manager;

pub use crate::backend_shared::ContainerShared;
pub use crate::backend_shared::ContainerSharedMutable;
pub use crate::backend_shared::BackendSharedReadonlyAtomic;
pub use crate::backend_shared::BackendSharedWritableAtomic;
pub use crate::backend_shared::BackendSharedReadonlyLockedAtomic;
pub use crate::backend_shared::BackendSharedWritableLockedAtomic;

pub use crate::backend::Container;
pub use crate::backend::BackendMemoryOnly;
pub use crate::backend::BackendReadonly;
pub use crate::backend::BackendWritable;
pub use crate::backend::BackendReadonlyLocked;
pub use crate::backend::BackendWritableLocked;

pub use crate::error::Error;

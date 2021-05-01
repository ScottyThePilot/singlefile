#![warn(missing_debug_implementations)]

extern crate serde;
pub extern crate serde_multi;
extern crate fs2;

mod error;
pub mod manager;

use serde::{Serialize, Deserialize};
use serde_multi::traits::SerdeStream;

pub use crate::error::Error;
use crate::manager::lock::FileLock;
pub use crate::manager::FileManager;
pub use crate::manager::lock::NoLock;
pub use crate::manager::lock::SharedLock;
pub use crate::manager::lock::ExclusiveLock;
use crate::manager::mode::FileMode;
pub use crate::manager::mode::Readonly;
pub use crate::manager::mode::Writable;
pub use crate::manager::mode::Reading;
pub use crate::manager::mode::Writing;
use crate::manager::*;

use std::path::Path;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Container<T, Manager> {
  item: T,
  manager: Manager
}

/// Type alias to a backend with no manager, only a value stored in memory.
pub type BackendMemoryOnly<T> = Container<T, ()>;
/// Type alias to a read-only, unlocked backend.
pub type BackendReadonly<T, Format> = Container<T, ManagerReadonly<Format>>;
/// Type alias to a readable and writable, unlocked backend.
pub type BackendWritable<T, Format> = Container<T, ManagerWritable<Format>>;
/// Type alias to a read-only, shared-locked backend.
pub type BackendReadonlyLocked<T, Format> = Container<T, ManagerReadonlyLocked<Format>>;
/// Type alias to a readable and writable, exclusively-locked backend.
pub type BackendWritableLocked<T, Format> = Container<T, ManagerWritableLocked<Format>>;

impl<T, Manager> Container<T, Manager> {
  #[inline(always)]
  pub fn into_inner(self) -> T {
    self.item
  }

  #[inline(always)]
  pub fn manager(&self) -> &Manager {
    &self.manager
  }

  #[inline(always)]
  pub fn borrow(&self) -> &T {
    &self.item
  }

  #[inline(always)]
  pub fn borrow_mut(&mut self) -> &mut T {
    &mut self.item
  }
}

impl<T> Container<T, ()> {
  #[inline(always)]
  pub fn new(item: T) -> Self {
    Container { item, manager: () }
  }
}

impl<T, Format, Lock, Mode> Container<T, FileManager<Format, Lock, Mode>>
where Format: SerdeStream, Lock: FileLock, Mode: FileMode<Format>, for<'de> T: Serialize + Deserialize<'de> {
  #[inline]
  pub fn open<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error>
  where Mode: Reading<T, Format> {
    let manager = FileManager::open(path, format)?;
    let item = manager.read()?;
    Ok(Container { item, manager })
  }

  #[inline]
  pub fn create_or<P: AsRef<Path>>(path: P, format: Format, item: T) -> Result<Self, Error> {
    let (item, manager) = FileManager::create_or(path, format, item)?;
    Ok(Container { item, manager })
  }

  #[inline]
  pub fn create_or_else<P: AsRef<Path>, C>(path: P, format: Format, closure: C) -> Result<Self, Error>
  where C: FnOnce() -> T {
    let (item, manager) = FileManager::create_or_else(path, format, closure)?;
    Ok(Container { item, manager })
  }

  #[inline]
  pub fn create_or_default<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error>
  where T: Default {
    let (item, manager) = FileManager::create_or_default(path, format)?;
    Ok(Container { item, manager })
  }
}

impl<T, Format, Lock, Mode> Container<T, FileManager<Format, Lock, Mode>>
where Format: SerdeStream {
  #[inline]
  pub fn refresh(&mut self) -> Result<(), Error>
  where Mode: Reading<T, Format> {
    self.manager.read().map(|item| self.item = item)
  }

  #[inline]
  pub fn commit(&self) -> Result<(), Error>
  where Mode: Writing<T, Format> {
    self.manager.write(&self.item)
  }

  #[inline]
  pub fn commit_insert(&mut self, item: T) -> Result<(), Error>
  where Mode: Writing<T, Format> {
    self.manager.write(&item)?;
    self.item = item;
    Ok(())
  }
}

impl<T, Manager> Deref for Container<T, Manager> {
  type Target = T;

  #[inline(always)]
  fn deref(&self) -> &T {
    self.borrow()
  }
}

impl<T, Manager> DerefMut for Container<T, Manager> {
  #[inline(always)]
  fn deref_mut(&mut self) -> &mut T {
    self.borrow_mut()
  }
}

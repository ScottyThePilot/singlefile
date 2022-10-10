use serde::{Serialize, Deserialize};

use crate::error::Error;
use crate::manager::lock::FileLock;
use crate::manager::mode::FileMode;
use crate::manager::*;

use std::path::Path;
use std::ops::{Deref, DerefMut};

/// Type alias to a container with no manager, only a value stored in memory.
pub type ContainerMemoryOnly<T> = Container<T, ()>;
/// Type alias to a read-only, unlocked container.
pub type ContainerReadonly<T, Format> = Container<T, ManagerReadonly<Format>>;
/// Type alias to a readable and writable, unlocked container.
pub type ContainerWritable<T, Format> = Container<T, ManagerWritable<Format>>;
/// Type alias to a read-only, shared-locked container.
pub type ContainerReadonlyLocked<T, Format> = Container<T, ManagerReadonlyLocked<Format>>;
/// Type alias to a readable and writable, exclusively-locked container.
pub type ContainerWritableLocked<T, Format> = Container<T, ManagerWritableLocked<Format>>;

#[derive(Debug)]
pub struct Container<T, Manager> {
  pub(crate) item: T,
  pub(crate) manager: Manager
}

impl<T, Manager> Container<T, Manager> {
  /// Extract the contained state.
  #[inline(always)]
  pub fn into_inner(self) -> T {
    self.item
  }

  /// Retrieve the contained file manager.
  #[inline(always)]
  pub const fn manager(&self) -> &Manager {
    &self.manager
  }

  #[inline(always)]
  pub const fn borrow(&self) -> &T {
    &self.item
  }

  #[inline(always)]
  pub fn borrow_mut(&mut self) -> &mut T {
    &mut self.item
  }
}

impl<T> Container<T, ()> {
  /// Create a new in-memory-only container, not connected to a file.
  #[inline(always)]
  pub const fn new(item: T) -> Self {
    Container { item, manager: () }
  }
}

impl<T, Format, Lock, Mode> Container<T, FileManager<Format, Lock, Mode>>
where Format: FileFormat, Lock: FileLock, Mode: FileMode<Format>, for<'de> T: Serialize + Deserialize<'de> {
  /// Opens a new [`Container`], returning an error if the file at the given path does not exist.
  pub fn open<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error>
  where Mode: Reading<T, Format> {
    let manager = FileManager::open(path, format)?;
    let item = manager.read()?;
    Ok(Container { item, manager })
  }

  /// Opens a new [`Container`], writing the given value to the file if it does not exist.
  pub fn create_or<P: AsRef<Path>>(path: P, format: Format, item: T) -> Result<Self, Error> {
    let (item, manager) = FileManager::create_or(path, format, item)?;
    Ok(Container { item, manager })
  }

  /// Opens a new [`Container`], writing the result of the given closure to the file if it does not exist.
  pub fn create_or_else<P: AsRef<Path>, C>(path: P, format: Format, closure: C) -> Result<Self, Error>
  where C: FnOnce() -> T {
    let (item, manager) = FileManager::create_or_else(path, format, closure)?;
    Ok(Container { item, manager })
  }

  /// Opens a new [`Container`], writing the default value of `T` to the file if it does not exist.
  pub fn create_or_default<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error>
  where T: Default {
    let (item, manager) = FileManager::create_or_default(path, format)?;
    Ok(Container { item, manager })
  }
}

impl<T, Format, Lock, Mode> Container<T, FileManager<Format, Lock, Mode>>
where Format: FileFormat {
  /// Reads a value from the managed file, replacing the current state in memory.
  pub fn refresh(&mut self) -> Result<(), Error>
  where Mode: Reading<T, Format> {
    self.manager.read().map(|item| self.item = item)
  }

  /// Writes the current in-memory state to the managed file.
  pub fn commit(&self) -> Result<(), Error>
  where Mode: Writing<T, Format> {
    self.manager.write(&self.item)
  }

  /// Writes a given state to the managed file, replacing the in-memory state.
  pub fn commit_with(&mut self, item: T) -> Result<(), Error>
  where Mode: Writing<T, Format> {
    self.item = item;
    self.manager.write(&self.item)
  }
}

impl<T, Manager> Deref for Container<T, Manager> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    self.borrow()
  }
}

impl<T, Manager> DerefMut for Container<T, Manager> {
  #[inline]
  fn deref_mut(&mut self) -> &mut T {
    self.borrow_mut()
  }
}

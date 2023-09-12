//! Container constructs providing single-ownership managed access to a file.

use crate::error::Error;
use crate::manager::lock::FileLock;
use crate::manager::mode::FileMode;
use crate::manager::*;

use std::io;
use std::ops::{Deref, DerefMut};
use std::path::Path;

/// Type alias to a container that is read-only.
pub type ContainerReadonly<T, Format> = Container<T, ManagerReadonly<Format>>;
/// Type alias to a container that is readable and writable.
pub type ContainerWritable<T, Format> = Container<T, ManagerWritable<Format>>;
/// Type alias to a container that is read-only, and has a shared file lock.
pub type ContainerReadonlyLocked<T, Format> = Container<T, ManagerReadonlyLocked<Format>>;
/// Type alias to a container that is readable and writable, and has an exclusive file lock.
pub type ContainerWritableLocked<T, Format> = Container<T, ManagerWritableLocked<Format>>;

/// A basic owned container allowing managed access to some underlying file.
#[derive(Debug)]
pub struct Container<T, Manager> {
  pub(crate) item: T,
  pub(crate) manager: Manager
}

impl<T, Manager> Container<T, Manager> {
  /// Extract the contained state.
  #[inline(always)]
  pub fn into_value(self) -> T {
    self.item
  }

  /// Extract the container manager.
  #[inline(always)]
  pub fn into_manager(self) -> Manager {
    self.manager
  }

  /// Gets a reference to the contained file manager.
  ///
  /// It is inadvisable to manipulate the manager manually.
  #[inline(always)]
  pub const fn manager(&self) -> &Manager {
    &self.manager
  }

  /// Gets a reference to the contained value.
  ///
  /// You may also operate on the container directly with [`Deref`] instead.
  #[inline(always)]
  pub const fn get(&self) -> &T {
    &self.item
  }

  /// Gets a mutable reference to the contained value.
  ///
  /// You may also operate on the container directly with [`DerefMut`] instead.
  #[inline(always)]
  pub fn get_mut(&mut self) -> &mut T {
    &mut self.item
  }
}

impl<T, Format, Lock, Mode> Container<T, FileManager<Format, Lock, Mode>>
where Format: FileFormat<T>, Lock: FileLock, Mode: FileMode<Format> {
  /// Opens a new [`Container`], returning an error if the file at the given path does not exist.
  pub fn open<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error<Format::FormatError>>
  where Mode: Reading<T, Format> {
    let manager = FileManager::open(path, format)?;
    let item = manager.read()?;
    Ok(Container { item, manager })
  }

  /// Opens a new [`Container`], creating a file at the given path if it does not exist, and overwriting its contents if it does.
  pub fn create_overwrite<P: AsRef<Path>>(path: P, format: Format, item: T) -> Result<Self, Error<Format::FormatError>> {
    let (item, manager) = FileManager::create_overwrite(path, format, item)?;
    Ok(Container { item, manager })
  }

  /// Opens a new [`Container`], writing the given value to the file if it does not exist.
  pub fn create_or<P: AsRef<Path>>(path: P, format: Format, item: T) -> Result<Self, Error<Format::FormatError>> {
    let (item, manager) = FileManager::create_or(path, format, item)?;
    Ok(Container { item, manager })
  }

  /// Opens a new [`Container`], writing the result of the given closure to the file if it does not exist.
  pub fn create_or_else<P: AsRef<Path>, C>(path: P, format: Format, closure: C) -> Result<Self, Error<Format::FormatError>>
  where C: FnOnce() -> T {
    let (item, manager) = FileManager::create_or_else(path, format, closure)?;
    Ok(Container { item, manager })
  }

  /// Opens a new [`Container`], writing the default value of `T` to the file if it does not exist.
  pub fn create_or_default<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error<Format::FormatError>>
  where T: Default {
    let (item, manager) = FileManager::create_or_default(path, format)?;
    Ok(Container { item, manager })
  }
}

impl<T, Format, Lock, Mode> Container<T, FileManager<Format, Lock, Mode>>
where Format: FileFormat<T> {
  /// Reads a value from the managed file, replacing the current state in memory.
  pub fn refresh(&mut self) -> Result<T, Error<Format::FormatError>>
  where Mode: Reading<T, Format> {
    self.manager.read().map(|item| std::mem::replace(&mut self.item, item))
  }

  /// Writes the current in-memory state to the managed file.
  pub fn commit(&self) -> Result<(), Error<Format::FormatError>>
  where Mode: Writing<T, Format> {
    self.manager.write(&self.item)
  }

  /// Writes the given state to the managed file, replacing the in-memory state.
  pub fn overwrite(&mut self, item: T) -> Result<(), Error<Format::FormatError>>
  where Mode: Writing<T, Format> {
    self.item = item;
    self.manager.write(&self.item)
  }
}

impl<T, Format, Lock, Mode> Container<T, FileManager<Format, Lock, Mode>>
where Lock: FileLock {
  /// Unlocks and closes this [`Container`], returning the contained state.
  pub fn close(self) -> io::Result<T> {
    self.manager.close().map(|()| self.item)
  }
}

impl<T, Manager> Deref for Container<T, Manager> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    self.get()
  }
}

impl<T, Manager> DerefMut for Container<T, Manager> {
  #[inline]
  fn deref_mut(&mut self) -> &mut T {
    self.get_mut()
  }
}

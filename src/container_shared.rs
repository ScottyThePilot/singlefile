use serde::{Serialize, Deserialize};

use crate::error::Error;
use crate::container::*;
use crate::manager::lock::FileLock;
use crate::manager::mode::FileMode;
use crate::manager::*;

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use std::path::Path;
use std::sync::Arc;

pub type AccessGuard<'a, T> = RwLockReadGuard<'a, T>;
pub type AccessGuardMut<'a, T> = RwLockWriteGuard<'a, T>;

/// Type alias to a shared, thread-safe, read-only, unlocked container.
pub type ContainerSharedReadonly<T, Format> = ContainerShared<T, ManagerReadonly<Format>>;
/// Type alias to a shared, thread-safe, readable and writable, unlocked container.
pub type ContainerSharedWritable<T, Format> = ContainerShared<T, ManagerWritable<Format>>;
/// Type alias to a shared, thread-safe, read-only, shared-locked container.
pub type ContainerSharedReadonlyLocked<T, Format> = ContainerShared<T, ManagerReadonlyLocked<Format>>;
/// Type alias to a shared, thread-safe, readable and writable, exclusively-locked container.
pub type ContainerSharedWritableLocked<T, Format> = ContainerShared<T, ManagerWritableLocked<Format>>;

/// A container that allows atomic reference-counted, mutable access (gated by an RwLock) to the underlying
/// file and contents. Cloning this container will not clone the underlying contents, it will clone the
/// underlying pointer, allowing multiple-access.
#[derive(Debug)]
#[repr(transparent)]
pub struct ContainerShared<T, Manager> {
  pub(crate) ptr: Arc<Container<RwLock<T>, Manager>>
}

impl<T, Manager> ContainerShared<T, Manager> {
  // Retrieve the contained file manager.
  #[inline(always)]
  pub fn manager(&self) -> &Manager {
    &self.ptr.manager
  }

  /// Gets immutable access to the underlying value `T`.
  #[inline]
  pub fn access(&self) -> AccessGuard<'_, T> {
    self.ptr.read()
  }

  /// Gets mutable access to the underlying value `T`.
  #[inline]
  pub fn access_mut(&self) -> AccessGuardMut<'_, T> {
    self.ptr.write()
  }

  /// Grants the caller immutable access to the underlying value `T`,
  /// but only for the duration of the provided function or closure.
  pub fn operate<F, R>(&self, operation: F) -> R
  where F: FnOnce(&T) -> R {
    operation(&*self.access())
  }

  /// Grants the caller mutable access to the underlying value `T`,
  /// but only for the duration of the provided function or closure.
  pub fn operate_mut<F, R>(&self, operation: F) -> R
  where F: FnOnce(&mut T) -> R {
    operation(&mut *self.access_mut())
  }
}

impl<T, Format, Lock, Mode> ContainerShared<T, FileManager<Format, Lock, Mode>>
where
  Format: FileFormat,
  Lock: FileLock,
  Mode: FileMode<Format>,
  for<'de> T: Serialize + Deserialize<'de>
{
  /// Opens a new [`ContainerSharedMutable`], returning an error if the file at the given path does not exist.
  #[inline]
  pub fn open<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error>
  where Mode: Reading<T, Format> {
    Container::<T, _>::open(path, format).map(From::from)
  }

  /// Opens a new [`ContainerSharedMutable`], writing the given value to the file if it does not exist.
  #[inline]
  pub fn create_or<P: AsRef<Path>>(path: P, format: Format, item: T) -> Result<Self, Error> {
    Container::<T, _>::create_or(path, format, item).map(From::from)
  }

  /// Opens a new [`ContainerSharedMutable`], writing the result of the given closure to the file if it does not exist.
  #[inline]
  pub fn create_or_else<P: AsRef<Path>, C>(path: P, format: Format, closure: C) -> Result<Self, Error>
  where C: FnOnce() -> T {
    Container::<T, _>::create_or_else(path, format, closure).map(From::from)
  }

  /// Opens a new [`ContainerSharedMutable`], writing the default value of `T` to the file if it does not exist.
  #[inline]
  pub fn create_or_default<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error>
  where T: Default {
    Container::<T, _>::create_or_default(path, format).map(From::from)
  }
}

impl<T, Format, Lock, Mode> ContainerShared<T, FileManager<Format, Lock, Mode>>
where Format: FileFormat {
  /// Reads a value from the managed file, replacing the current state in memory,
  /// immediately granting the caller immutable access to that state
  /// for the duration of the provided function or closure.
  ///
  /// The provided closure takes (1) a reference to the new state, and (2) the old state.
  pub fn operate_refresh<F, R>(&self, operation: F) -> Result<R, Error>
  where Mode: Reading<T, Format>, F: FnOnce(&T, T) -> R {
    let mut guard = self.access_mut();
    let item = self.ptr.manager.read()?;
    let old_item = std::mem::replace(&mut *guard, item);
    Ok(operation(&guard, old_item))
  }

  /// Grants the caller mutable access to the underlying value `T`,
  /// but only for the duration of the provided function or closure,
  /// immediately committing any changes made.
  pub fn operate_mut_commit<F, R>(&self, operation: F) -> Result<R, Error>
  where Mode: Writing<T, Format>, F: FnOnce(&mut T) -> R {
    let mut guard = self.access_mut();
    let ret = operation(&mut guard);
    self.commit_guard(RwLockWriteGuard::downgrade(guard))
      .map(|()| ret)
  }

  /// Reads a value from the managed file, replacing the current state in memory.
  ///
  /// Returns the value of the previous state if the operation succeeded.
  pub fn refresh(&self) -> Result<T, Error>
  where Mode: Reading<T, Format> {
    let mut guard = self.access_mut();
    let item = self.ptr.manager.read()?;
    let old_item = std::mem::replace(&mut *guard, item);
    Ok(old_item)
  }

  /// Writes the current in-memory state to the managed file.
  pub fn commit(&self) -> Result<(), Error>
  where Mode: Writing<T, Format> {
    let guard = self.access();
    self.commit_guard(guard)
  }

  pub fn commit_guard(&self, guard: AccessGuard<'_, T>) -> Result<(), Error>
  where Mode: Writing<T, Format> {
    self.ptr.manager.write(&*guard)
  }
}

impl<T, Manager> Clone for ContainerShared<T, Manager> {
  #[inline]
  fn clone(&self) -> Self {
    ContainerShared { ptr: Arc::clone(&self.ptr) }
  }
}

impl<T, Manager> From<Container<T, Manager>> for ContainerShared<T, Manager> {
  #[inline]
  fn from(container: Container<T, Manager>) -> Self {
    ContainerShared {
      ptr: Arc::new(Container {
        item: RwLock::new(container.item),
        manager: container.manager
      })
    }
  }
}

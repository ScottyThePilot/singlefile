//! Container constructs allowing multiple-ownership managed access to a file.

use crate::error::{Error, UserError};
use crate::container::*;
use crate::manager::lock::FileLock;
use crate::manager::mode::FileMode;
use crate::manager::*;

pub extern crate parking_lot;

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use std::path::Path;
use std::sync::Arc;

/// An alias to [`parking_lot::RwLockReadGuard`].
pub type AccessGuard<'a, T> = RwLockReadGuard<'a, T>;
/// An alias to [`parking_lot::RwLockWriteGuard`].
pub type AccessGuardMut<'a, T> = RwLockWriteGuard<'a, T>;

/// Type alias to a shared, thread-safe container that is read-only.
pub type ContainerSharedReadonly<T, Format> = ContainerShared<T, ManagerReadonly<Format>>;
/// Type alias to a shared, thread-safe container that is readable and writable.
pub type ContainerSharedWritable<T, Format> = ContainerShared<T, ManagerWritable<Format>>;
/// Type alias to a shared, thread-safe container that is read-only, and has a shared file lock.
pub type ContainerSharedReadonlyLocked<T, Format> = ContainerShared<T, ManagerReadonlyLocked<Format>>;
/// Type alias to a shared, thread-safe container that is readable and writable, and has an exclusive file lock.
pub type ContainerSharedWritableLocked<T, Format> = ContainerShared<T, ManagerWritableLocked<Format>>;

/// A container that allows atomic reference-counted, mutable access (gated by an RwLock) to the underlying
/// file and contents. Cloning this container will not clone the underlying contents, it will clone the
/// underlying pointer, allowing multiple-access.
#[derive(Debug)]
#[repr(transparent)]
pub struct ContainerShared<T, Manager> {
  ptr: Arc<Container<RwLock<T>, Manager>>
}

impl<T, Manager> ContainerShared<T, Manager> {
  /// Try to extract the contained state.
  pub fn try_into_value(self) -> Result<T, Self> {
    match Arc::try_unwrap(self.ptr) {
      Ok(container) => Ok(container.into_value().into_inner()),
      Err(ptr) => Err(ContainerShared { ptr })
    }
  }

  /// Try to extract the contained state.
  pub fn try_into_manager(self) -> Result<Manager, Self> {
    match Arc::try_unwrap(self.ptr) {
      Ok(container) => Ok(container.into_manager()),
      Err(ptr) => Err(ContainerShared { ptr })
    }
  }

  /// Gets a reference to the contained file manager.
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

  /// Attempts to convert this [`ContainerShared`] into an owned [`Container`],
  /// as long as this shared container is the only instance in existence.
  pub fn try_unwrap(self) -> Result<Container<T, Manager>, Self> {
    match Arc::try_unwrap(self.ptr) {
      Ok(ptr) => Ok(Container {
        item: ptr.item.into_inner(),
        manager: ptr.manager
      }),
      Err(ptr) => Err(ContainerShared { ptr })
    }
  }
}

impl<T, Format, Lock, Mode> ContainerShared<T, FileManager<Format, Lock, Mode>>
where
  Format: FileFormat<T>,
  Lock: FileLock,
  Mode: FileMode<Format>
{
  /// Opens a new [`ContainerShared`], returning an error if the file at the given path does not exist.
  #[inline]
  pub fn open<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error<Format::FormatError>>
  where Mode: Reading<T, Format> {
    Container::<T, _>::open(path, format).map(From::from)
  }

  /// Opens a new [`ContainerShared`], writing the given value to the file if it does not exist.
  #[inline]
  pub fn create_or<P: AsRef<Path>>(path: P, format: Format, item: T) -> Result<Self, Error<Format::FormatError>> {
    Container::<T, _>::create_or(path, format, item).map(From::from)
  }

  /// Opens a new [`ContainerShared`], writing the result of the given closure to the file if it does not exist.
  #[inline]
  pub fn create_or_else<P: AsRef<Path>, C>(path: P, format: Format, closure: C) -> Result<Self, Error<Format::FormatError>>
  where C: FnOnce() -> T {
    Container::<T, _>::create_or_else(path, format, closure).map(From::from)
  }

  /// Opens a new [`ContainerShared`], writing the default value of `T` to the file if it does not exist.
  #[inline]
  pub fn create_or_default<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error<Format::FormatError>>
  where T: Default {
    Container::<T, _>::create_or_default(path, format).map(From::from)
  }
}

impl<T, Format, Lock, Mode> ContainerShared<T, FileManager<Format, Lock, Mode>>
where Format: FileFormat<T> {
  /// Reads a value from the managed file, replacing the current state in memory,
  /// immediately granting the caller immutable access to that state
  /// for the duration of the provided function or closure.
  ///
  /// The provided closure takes (1) a reference to the new state, and (2) the old state.
  ///
  /// This function acquires a mutable lock on the shared state.
  pub fn operate_refresh<F, R>(&self, operation: F) -> Result<R, Error<Format::FormatError>>
  where Mode: Reading<T, Format>, F: FnOnce(&T, T) -> R {
    let mut guard = self.access_mut();
    let item = self.ptr.manager.read()?;
    let old_item = std::mem::replace(&mut *guard, item);
    Ok(operation(&guard, old_item))
  }

  /// Grants the caller mutable access to the underlying value `T`,
  /// but only for the duration of the provided function or closure,
  /// immediately committing any changes made.
  ///
  /// This function acquires a mutable lock on the shared state.
  pub fn operate_mut_commit<F, R, U>(&self, operation: F) -> Result<R, UserError<Format::FormatError, U>>
  where Mode: Writing<T, Format>, F: FnOnce(&mut T) -> Result<R, U> {
    let mut guard = self.access_mut();
    let ret = operation(&mut guard).map_err(UserError::User)?;
    self.commit_guard(downgrade(guard))?;
    Ok(ret)
  }

  /// Reads a value from the managed file, replacing the current state in memory.
  ///
  /// Returns the value of the previous state if the operation succeeded.
  ///
  /// This function acquires an immutable lock on the shared state.
  pub fn refresh(&self) -> Result<T, Error<Format::FormatError>>
  where Mode: Reading<T, Format> {
    let mut guard = self.access_mut();
    let item = self.ptr.manager.read()?;
    let old_item = std::mem::replace(&mut *guard, item);
    Ok(old_item)
  }

  /// Writes the current in-memory state to the managed file.
  ///
  /// This function acquires an immutable lock on the shared state.
  /// Don't call this if you currently have an access guard, use [`ContainerShared::commit_guard`] instead.
  pub fn commit(&self) -> Result<(), Error<Format::FormatError>>
  where Mode: Writing<T, Format> {
    let guard = self.access();
    self.commit_guard(guard)
  }

  /// Writes to the managed file given an access guard.
  pub fn commit_guard(&self, guard: AccessGuard<'_, T>) -> Result<(), Error<Format::FormatError>>
  where Mode: Writing<T, Format> {
    self.ptr.manager.write(&*guard)
  }

  /// Writes the given state to the managed file, replacing the in-memory state.
  pub fn overwrite(&self, item: T) -> Result<(), Error<Format::FormatError>>
  where Mode: Writing<T, Format> {
    let mut guard = self.access_mut();
    *guard = item;
    self.commit_guard(downgrade(guard))
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

/// Downgrades an exclusive mutable guard to an immutable guard.
///
/// Essentially an alias to [`parking_lot::RwLockWriteGuard::downgrade`].
pub fn downgrade<'a, T>(guard: AccessGuardMut<'a, T>) -> AccessGuard<'a, T> {
  AccessGuardMut::downgrade(guard)
}

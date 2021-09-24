use serde::{Serialize, Deserialize};

use crate::error::Error;
use crate::backend::*;
use crate::manager::lock::FileLock;
use crate::manager::mode::FileMode;
use crate::manager::*;

use std::path::Path;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::ops::Deref;
use std::sync::Arc;

pub type AccessGuard<'a, T> = RwLockReadGuard<'a, T>;
pub type AccessGuardMut<'a, T> = RwLockWriteGuard<'a, T>;

/// Type alias to a shared, thread-safe, read-only, unlocked backend.
pub type BackendSharedReadonlyAtomic<T, Format> = ContainerShared<T, ManagerReadonly<Format>>;
/// Type alias to a shared, thread-safe, readable and writable, unlocked backend.
pub type BackendSharedWritableAtomic<T, Format> = ContainerSharedMutable<T, ManagerWritable<Format>>;
/// Type alias to a shared, thread-safe, read-only, shared-locked backend.
pub type BackendSharedReadonlyLockedAtomic<T, Format> = ContainerShared<T, ManagerReadonlyLocked<Format>>;
/// Type alias to a shared, thread-safe, readable and writable, exclusively-locked backend.
pub type BackendSharedWritableLockedAtomic<T, Format> = ContainerSharedMutable<T, ManagerWritableLocked<Format>>;

/// A container that allows atomic reference counted, immutable access to the underlying file and contents.
/// Cloning this container will not clone the underlying contents, it will clone the underlying pointer,
/// allowing multiple-access.
#[derive(Debug)]
#[repr(transparent)]
pub struct ContainerShared<T, Manager> {
  pub(crate) ptr: Arc<Container<T, Manager>>
}

impl<Manager, T> ContainerShared<T, Manager> {
  /// Attempts to unwrap the contained state if only one reference exists, other wise it returns `Self`.
  pub fn try_into_inner(self) -> Result<T, Self> {
    match Arc::try_unwrap(self.ptr) {
      Ok(container) => Ok(container.item),
      Err(ptr) => Err(ContainerShared { ptr })
    }
  }

  // Retrieve the contained file manager.
  #[inline(always)]
  pub fn manager(&self) -> &Manager {
    &self.ptr.manager
  }

  #[inline(always)]
  pub fn borrow(&self) -> &T {
    &self.ptr
  }

  #[inline]
  pub fn try_borrow_mut(&mut self) -> Option<&mut T> {
    Arc::get_mut(&mut self.ptr).map(|ptr| &mut ptr.item)
  }
}

impl<T, Format, Lock, Mode> ContainerShared<T, FileManager<Format, Lock, Mode>>
where Format: FileFormat, Lock: FileLock, Mode: FileMode<Format>, for<'de> T: Serialize + Deserialize<'de> {
  /// Opens a new [`ContainerShared`], returning an error if the file at the given path does not exist.
  pub fn open<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error>
  where Mode: Reading<T, Format> {
    Ok(ContainerShared {
      ptr: Arc::new(Container::open(path, format)?)
    })
  }

  /// Opens a new [`ContainerShared`], writing the given value to the file if it does not exist.
  pub fn create_or<P: AsRef<Path>>(path: P, format: Format, item: T) -> Result<Self, Error> {
    Ok(ContainerShared {
      ptr: Arc::new(Container::create_or(path, format, item)?)
    })
  }

  /// Opens a new [`ContainerShared`], writing the result of the given closure to the file if it does not exist.
  pub fn create_or_else<P: AsRef<Path>, C>(path: P, format: Format, closure: C) -> Result<Self, Error>
  where C: FnOnce() -> T {
    Ok(ContainerShared {
      ptr: Arc::new(Container::create_or_else(path, format, closure)?)
    })
  }

  /// Opens a new [`ContainerShared`], writing the default value of `T` to the file if it does not exist.
  pub fn create_or_default<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error>
  where T: Default {
    Ok(ContainerShared {
      ptr: Arc::new(Container::create_or_default(path, format)?)
    })
  }
}

impl<T, Manager> Clone for ContainerShared<T, Manager> {
  fn clone(&self) -> Self {
    ContainerShared { ptr: Arc::clone(&self.ptr) }
  }
}

impl<Manager, T> Deref for ContainerShared<T, Manager> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    self.borrow()
  }
}

/// A container that allows atomic reference-counted, mutable access (gated by an RwLock) to the underlying
/// file and contents. Cloning this container will not clone the underlying contents, it will clone the
/// underlying pointer, allowing multiple-access.
#[derive(Debug)]
#[repr(transparent)]
pub struct ContainerSharedMutable<T, Manager> {
  pub(crate) ptr: Arc<Container<RwLock<T>, Manager>>
}

impl<Manager, T> ContainerSharedMutable<T, Manager> {
  // Retrieve the contained file manager.
  #[inline(always)]
  pub fn manager(&self) -> &Manager {
    &self.ptr.manager
  }

  /// Gets immutable access to the underlying value `T`.
  #[inline]
  pub fn access(&self) -> AccessGuard<'_, T> {
    self.ptr.read().unwrap()
  }

  /// Gets mutable access to the underlying value `T`.
  #[inline]
  pub fn access_mut(&self) -> AccessGuardMut<'_, T> {
    self.ptr.write().unwrap()
  }
}

impl<T, Format, Lock, Mode> ContainerSharedMutable<T, FileManager<Format, Lock, Mode>>
where Format: FileFormat, Lock: FileLock, Mode: FileMode<Format>, for<'de> T: Serialize + Deserialize<'de> {
  /// Opens a new [`ContainerSharedMutable`], returning an error if the file at the given path does not exist.
  pub fn open<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error>
  where Mode: Reading<T, Format> {
    let manager = FileManager::open(path, format)?;
    let item = RwLock::new(manager.read()?);
    let ptr = Arc::new(Container { item, manager });
    Ok(ContainerSharedMutable { ptr })
  }

  /// Opens a new [`ContainerSharedMutable`], writing the given value to the file if it does not exist.
  pub fn create_or<P: AsRef<Path>>(path: P, format: Format, item: T) -> Result<Self, Error> {
    let (item, manager) = FileManager::create_or(path, format, item)?;
    let ptr = Arc::new(Container { item: RwLock::new(item), manager });
    Ok(ContainerSharedMutable { ptr })
  }

  /// Opens a new [`ContainerSharedMutable`], writing the result of the given closure to the file if it does not exist.
  pub fn create_or_else<P: AsRef<Path>, C>(path: P, format: Format, closure: C) -> Result<Self, Error>
  where C: FnOnce() -> T {
    let (item, manager) = FileManager::create_or_else(path, format, closure)?;
    let ptr = Arc::new(Container { item: RwLock::new(item), manager });
    Ok(ContainerSharedMutable { ptr })
  }

  /// Opens a new [`ContainerSharedMutable`], writing the default value of `T` to the file if it does not exist.
  pub fn create_or_default<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error>
  where T: Default {
    let (item, manager) = FileManager::create_or_default(path, format)?;
    let ptr = Arc::new(Container { item: RwLock::new(item), manager });
    Ok(ContainerSharedMutable { ptr })
  }
}

impl<T, Format, Lock, Mode> ContainerSharedMutable<T, FileManager<Format, Lock, Mode>>
where Format: FileFormat {
  /// Reads a value from the managed file, replacing the current state in memory.
  pub fn refresh(&self) -> Result<(), Error>
  where Mode: Reading<T, Format> {
    let mut inner = self.access_mut();
    self.ptr.manager.read().map(|item| *inner = item)
  }

  /// Writes the current in-memory state to the managed file.
  pub fn commit(&self) -> Result<(), Error>
  where Mode: Writing<T, Format> {
    let inner = self.access();
    self.ptr.manager.write(&*inner)
  }

  /// Writes a given state to the managed file, replacing the in-memory state.
  pub fn commit_with(&self, item: T) -> Result<(), Error>
  where Mode: Writing<T, Format> {
    let mut inner = self.access_mut();
    *inner = item;
    self.ptr.manager.write(&*inner)
  }
}

impl<T, Manager> Clone for ContainerSharedMutable<T, Manager> {
  #[inline]
  fn clone(&self) -> Self {
    ContainerSharedMutable { ptr: Arc::clone(&self.ptr) }
  }
}

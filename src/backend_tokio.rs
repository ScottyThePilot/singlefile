use serde::{Serialize, Deserialize};

use crate::error::Error;
use crate::backend::*;
use crate::manager::lock::FileLock;
use crate::manager::mode::FileMode;
use crate::manager::*;

use tokio::sync::RwLock;
use tokio::sync::RwLockReadGuard;
use tokio::sync::RwLockWriteGuard;
use tokio::sync::OwnedRwLockReadGuard;
use tokio::sync::OwnedRwLockWriteGuard;
use tokio::task::spawn_blocking;

use std::path::Path;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Type alias to a shared, thread-safe, read-only, unlocked backend.
pub type BackendSharedReadonlyAtomic<T, Format> = ContainerAsync<T, ManagerReadonly<Format>>;
/// Type alias to a shared, thread-safe, readable and writable, unlocked backend.
pub type BackendSharedWritableAtomic<T, Format> = ContainerAsync<T, ManagerWritable<Format>>;
/// Type alias to a shared, thread-safe, read-only, shared-locked backend.
pub type BackendSharedReadonlyLockedAtomic<T, Format> = ContainerAsync<T, ManagerReadonlyLocked<Format>>;
/// Type alias to a shared, thread-safe, readable and writable, exclusively-locked backend.
pub type BackendSharedWritableLockedAtomic<T, Format> = ContainerAsync<T, ManagerWritableLocked<Format>>;

/// A container that allows atomic reference-counted, mutable access (gated by an RwLock) to the underlying
/// file and contents. Cloning this container will not clone the underlying contents, it will clone the
/// underlying pointer, allowing multiple-access.
#[derive(Debug)]
#[repr(transparent)]
pub struct ContainerAsync<T, Manager> {
  pub(crate) ptr: Arc<RwLock<Container<T, Manager>>>
}

impl<Manager, T> ContainerAsync<T, Manager> {
  /// Gets immutable access to the underlying value `T`.
  #[inline]
  pub async fn access(&self) -> AccessGuard<'_, T, Manager> {
    AccessGuard { inner: self.ptr.read().await }
  }

  /// Gets mutable access to the underlying value `T`.
  #[inline]
  pub async fn access_mut(&self) -> AccessGuardMut<'_, T, Manager> {
    AccessGuardMut { inner: self.ptr.write().await }
  }

  /// Gets immutable access to the underlying value `T`.
  #[inline]
  pub async fn access_owned(self) -> OwnedAccessGuard<T, Manager> {
    OwnedAccessGuard { inner: self.ptr.read_owned().await }
  }

  /// Gets mutable access to the underlying value `T`.
  #[inline]
  pub async fn access_owned_mut(self) -> OwnedAccessGuardMut<T, Manager> {
    OwnedAccessGuardMut { inner: self.ptr.write_owned().await }
  }
}

impl<T, Format, Lock, Mode> ContainerAsync<T, FileManager<Format, Lock, Mode>>
where
  Format: FileFormat + Send + 'static,
  Lock: FileLock + Send + 'static,
  Mode: FileMode<Format> + Send + 'static,
  for<'de> T: Serialize + Deserialize<'de> + Send + 'static
{
  /// Opens a new [`ContainerSharedMutable`], returning an error if the file at the given path does not exist.
  pub async fn open<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error>
  where Mode: Reading<T, Format> {
    let path = path.as_ref().to_owned();
    let container = spawn_blocking(move || {
      Container::<T, _>::open(path, format)
    }).await.unwrap()?;

    Ok(ContainerAsync {
      ptr: Arc::new(RwLock::new(container))
    })
  }

  /// Opens a new [`ContainerSharedMutable`], writing the given value to the file if it does not exist.
  pub async fn create_or<P: AsRef<Path>>(path: P, format: Format, item: T) -> Result<Self, Error> {
    let path = path.as_ref().to_owned();
    let container = spawn_blocking(move || {
      Container::<T, _>::create_or(path, format, item)
    }).await.unwrap()?;

    Ok(ContainerAsync {
      ptr: Arc::new(RwLock::new(container))
    })
  }

  /// Opens a new [`ContainerSharedMutable`], writing the result of the given closure to the file if it does not exist.
  pub async fn create_or_else<P: AsRef<Path>, C>(path: P, format: Format, closure: C) -> Result<Self, Error>
  where C: FnOnce() -> T + Send + 'static {
    let path = path.as_ref().to_owned();
    let container = spawn_blocking(move || {
      Container::<T, _>::create_or_else(path, format, closure)
    }).await.unwrap()?;

    Ok(ContainerAsync {
      ptr: Arc::new(RwLock::new(container))
    })
  }

  /// Opens a new [`ContainerSharedMutable`], writing the default value of `T` to the file if it does not exist.
  pub async fn create_or_default<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error>
  where T: Default {
    let path = path.as_ref().to_owned();
    let container = spawn_blocking(move || {
      Container::<T, _>::create_or_default(path, format)
    }).await.unwrap()?;

    Ok(ContainerAsync {
      ptr: Arc::new(RwLock::new(container))
    })
  }
}

impl<T, Format, Lock, Mode> ContainerAsync<T, FileManager<Format, Lock, Mode>>
where
  Format: FileFormat + Send + Sync + 'static,
  Lock: Send + Sync + 'static,
  Mode: Send + Sync + 'static,
  T: Send + Sync + 'static
{
  /// Reads a value from the managed file, replacing the current state in memory.
  pub async fn refresh(&self) -> Result<(), Error>
  where Mode: Reading<T, Format> {
    let mut lock = self.ptr.clone().write_owned().await;
    spawn_blocking(move || {
      lock.manager.read().map(|item| lock.item = item)
    }).await.unwrap()
  }

  /// Writes the current in-memory state to the managed file.
  pub async fn commit(&self) -> Result<(), Error>
  where Mode: Writing<T, Format> {
    let lock = self.ptr.clone().read_owned().await;
    spawn_blocking(move || {
      lock.manager.write(&lock.item)
    }).await.unwrap()
  }

  /// Writes a given state to the managed file, replacing the in-memory state.
  pub async fn commit_with(&self, item: T) -> Result<(), Error>
  where Mode: Writing<T, Format> {
    let mut lock = self.ptr.clone().write_owned().await;
    lock.item = item;
    spawn_blocking(move || {
      lock.manager.write(&lock.item)
    }).await.unwrap()
  }
}

impl<T, Manager> Clone for ContainerAsync<T, Manager> {
  #[inline]
  fn clone(&self) -> Self {
    ContainerAsync { ptr: Arc::clone(&self.ptr) }
  }
}



macro_rules! impl_deref {
  (
    $Struct:ident $(< $($lt:lifetime),* $(,)? $($g:ident),* $(,)? >)?,
    $Target:ty, $this:ident, $deref:expr, $deref_mut:expr
  ) => {
    impl_deref!($Struct $(< $($lt,)* $($g,)* >)?, $Target, $this, $deref);

    impl $(< $($lt,)* $($g,)* >)? DerefMut for $Struct $(< $($lt,)* $($g,)* >)? {
      #[inline]
      fn deref_mut(&mut self) -> &mut Self::Target {
        let $this = self;
        $deref_mut
      }
    }
  };
  (
    $Struct:ident $(< $($lt:lifetime),* $(,)? $($g:ident),* $(,)? >)?,
    $Target:ty, $this:ident, $deref:expr
  ) => {
    impl $(< $($lt,)* $($g,)* >)? Deref for $Struct $(< $($lt,)* $($g,)* >)? {
      type Target = $Target;

      #[inline]
      fn deref(&self) -> &Self::Target {
        let $this = self;
        $deref
      }
    }
  };
}

macro_rules! define_guard {
  ($vis:vis struct $Struct:ident $(< $($lt:lifetime),* $(,)? $($g:ident),* $(,)? >)?, $Inner:ty) => {
    #[repr(transparent)]
    #[derive(Debug)]
    $vis struct $Struct $(< $($lt,)* $($g,)* >)? {
      pub inner: $Inner
    }
  };
}

define_guard!(pub struct AccessGuard<'a, T, Manager>, RwLockReadGuard<'a, Container<T, Manager>>);
impl_deref!(AccessGuard<'a, T, Manager>, T, this, &this.inner.item);
define_guard!(pub struct AccessGuardMut<'a, T, Manager>, RwLockWriteGuard<'a, Container<T, Manager>>);
impl_deref!(AccessGuardMut<'a, T, Manager>, T, this, &this.inner.item, &mut this.inner.item);
define_guard!(pub struct OwnedAccessGuard<T, Manager>, OwnedRwLockReadGuard<Container<T, Manager>>);
impl_deref!(OwnedAccessGuard<T, Manager>, T, this, &this.inner.item);
define_guard!(pub struct OwnedAccessGuardMut<T, Manager>, OwnedRwLockWriteGuard<Container<T, Manager>>);
impl_deref!(OwnedAccessGuardMut<T, Manager>, T, this, &this.inner.item, &mut this.inner.item);

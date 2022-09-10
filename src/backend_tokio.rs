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
use std::sync::Arc;

pub type AccessGuard<'a, T> = RwLockReadGuard<'a, T>;
pub type AccessGuardMut<'a, T> = RwLockWriteGuard<'a, T>;
pub type OwnedAccessGuard<T> = OwnedRwLockReadGuard<T>;
pub type OwnedAccessGuardMut<T> = OwnedRwLockWriteGuard<T>;

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
/// underlying pointers, allowing multiple-access.
#[derive(Debug)]
pub struct ContainerAsync<T, Manager> {
  pub(crate) item: Arc<RwLock<T>>,
  pub(crate) manager: Arc<Manager>
}

impl<T, Manager> ContainerAsync<T, Manager> {
  /// Try to extract the contained state.
  pub fn try_into_inner(self) -> Result<T, Self> {
    let ContainerAsync { item, manager } = self;
    match Arc::try_unwrap(item) {
      Ok(item) => Ok(item.into_inner()),
      Err(item) => Err(ContainerAsync { item, manager })
    }
  }

  /// Retrieve the contained file manager.
  #[inline]
  pub fn manager(&self) -> &Manager {
    &self.manager
  }

  /// Gets immutable access to the underlying value `T`.
  #[inline]
  pub async fn access(&self) -> AccessGuard<'_, T> {
    self.item.read().await
  }

  /// Gets mutable access to the underlying value `T`.
  #[inline]
  pub async fn access_mut(&self) -> AccessGuardMut<'_, T> {
    self.item.write().await
  }

  /// Gets immutable access to the underlying value `T`.
  #[inline]
  pub async fn access_owned(&self) -> OwnedAccessGuard<T> {
    self.item.clone().read_owned().await
  }

  /// Gets mutable access to the underlying value `T`.
  #[inline]
  pub async fn access_owned_mut(&self) -> OwnedAccessGuardMut<T> {
    self.item.clone().write_owned().await
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
    spawn_blocking(move || Container::<T, _>::open(path, format))
      .await.unwrap().map(From::from)
  }

  /// Opens a new [`ContainerSharedMutable`], writing the given value to the file if it does not exist.
  pub async fn create_or<P: AsRef<Path>>(path: P, format: Format, item: T) -> Result<Self, Error> {
    let path = path.as_ref().to_owned();
    spawn_blocking(move || Container::<T, _>::create_or(path, format, item))
      .await.unwrap().map(From::from)
  }

  /// Opens a new [`ContainerSharedMutable`], writing the result of the given closure to the file if it does not exist.
  pub async fn create_or_else<P: AsRef<Path>, C>(path: P, format: Format, closure: C) -> Result<Self, Error>
  where C: FnOnce() -> T + Send + 'static {
    let path = path.as_ref().to_owned();
    spawn_blocking(move || Container::<T, _>::create_or_else(path, format, closure))
      .await.unwrap().map(From::from)
  }

  /// Opens a new [`ContainerSharedMutable`], writing the default value of `T` to the file if it does not exist.
  pub async fn create_or_default<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error>
  where T: Default {
    let path = path.as_ref().to_owned();
    spawn_blocking(move || Container::<T, _>::create_or_default(path, format))
      .await.unwrap().map(From::from)
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
    let mut lock = self.item.clone().write_owned().await;
    let manager = self.manager.clone();
    spawn_blocking(move || manager.read().map(|item| *lock = item))
      .await.unwrap()
  }

  /// Writes the current in-memory state to the managed file.
  pub async fn commit(&self) -> Result<(), Error>
  where Mode: Writing<T, Format> {
    let lock = self.item.clone().read_owned().await;
    let manager = self.manager.clone();
    spawn_blocking(move || manager.write(&*lock))
      .await.unwrap()
  }

  /// Writes a given state to the managed file, replacing the in-memory state.
  pub async fn commit_with(&self, item: T) -> Result<(), Error>
  where Mode: Writing<T, Format> {
    let mut lock = self.item.clone().write_owned().await;
    *lock = item;
    let manager = self.manager.clone();
    spawn_blocking(move || manager.write(&*lock))
      .await.unwrap()
  }
}

impl<T, Manager> Clone for ContainerAsync<T, Manager> {
  #[inline]
  fn clone(&self) -> Self {
    ContainerAsync {
      item: Arc::clone(&self.item),
      manager: Arc::clone(&self.manager)
    }
  }
}

impl<T, Manager> From<Container<T, Manager>> for ContainerAsync<T, Manager> {
  fn from(container: Container<T, Manager>) -> Self {
    ContainerAsync {
      item: Arc::new(RwLock::new(container.item)),
      manager: Arc::new(container.manager)
    }
  }
}

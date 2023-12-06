//! Container constructs allowing multiple-ownership, asynchronous, managed access to a file.
//!
//! This module can be enabled with the `shared-async` cargo feature.

mod guards;

use crate::error::{Error, UserError};
use crate::container::*;
use crate::manager::lock::FileLock;
use crate::manager::mode::FileMode;
use crate::manager::*;

pub use self::guards::{
  AccessGuard,
  AccessGuardMut,
  OwnedAccessGuard,
  OwnedAccessGuardMut
};

use tokio::sync::RwLock;

use std::path::Path;
use std::sync::Arc;

/// Type alias to a shared, asynchronous, thread-safe container that is read-only.
pub type ContainerSharedAsyncReadonly<T, Format> = ContainerSharedAsync<T, ManagerReadonly<Format>>;
/// Type alias to a shared, asynchronous, thread-safe container that is readable and writable.
pub type ContainerSharedAsyncWritable<T, Format> = ContainerSharedAsync<T, ManagerWritable<Format>>;
/// Type alias to a shared, asynchronous, thread-safe container that is readable and writable (with atomic writes).
/// See [`Atomic`] for more information.
pub type ContainerSharedAsyncAtomic<T, Format> = ContainerSharedAsync<T, ManagerAtomic<Format>>;
/// Type alias to a shared, asynchronous, thread-safe container that is read-only, and has a shared file lock.
pub type ContainerSharedAsyncReadonlyLocked<T, Format> = ContainerSharedAsync<T, ManagerReadonlyLocked<Format>>;
/// Type alias to a shared, asynchronous, thread-safe container that is readable and writable, and has an exclusive file lock.
pub type ContainerSharedAsyncWritableLocked<T, Format> = ContainerSharedAsync<T, ManagerWritableLocked<Format>>;
/// Type alias to a shared, asynchronous, thread-safe container that is readable and writable (with atomic writes), and has an exclusive file lock.
/// See [`Atomic`] for more information.
pub type ContainerSharedAsyncAtomicLocked<T, Format> = ContainerSharedAsync<T, ManagerAtomicLocked<Format>>;

macro_rules! spawn_blocking {
  ($expr:expr) => (tokio::task::spawn_blocking(move || $expr).await.expect("blocking task failed"));
}

/// A container that allows asynchronous atomic reference-counted, mutable access (gated by an [`RwLock`]) to the
/// underlying file and contents. Cloning this container will not clone the underlying contents, it will clone the
/// underlying pointer, allowing multiple-access.
#[repr(transparent)]
#[derive(Debug)]
pub struct ContainerSharedAsync<T, Manager> {
  ptr: Arc<RwLock<Container<T, Manager>>>
}

impl<T, Manager> ContainerSharedAsync<T, Manager> {
  /// Create a new [`ContainerSharedAsync`] from the value and manager directly.
  pub fn new(value: T, manager: Manager) -> Self {
    ContainerSharedAsync::from(Container::new(value, manager))
  }

  /// Returns the inner owned [`Container`], as long as there are no other existing pointers.
  /// Otherwise, the same [`ContainerSharedAsync`] is returned back.
  pub fn try_unwrap(self) -> Result<Container<T, Manager>, Self> {
    match Arc::try_unwrap(self.ptr) {
      Ok(inner) => Ok(RwLock::into_inner(inner)),
      Err(ptr) => Err(ContainerSharedAsync { ptr })
    }
  }

  /// Returns a mutable reference into the inner [`Container`], as long as there are no other existing pointers.
  pub fn get_mut(&mut self) -> Option<&mut Container<T, Manager>> {
    Arc::get_mut(&mut self.ptr).map(RwLock::get_mut)
  }

  /// Gets immutable access to the underlying container and value `T`.
  #[inline]
  pub async fn access(&self) -> AccessGuard<'_, T, Manager> {
    AccessGuard::new(self.ptr.read().await)
  }

  /// Gets mutable access to the underlying container and value `T`.
  #[inline]
  pub async fn access_mut(&self) -> AccessGuardMut<'_, T, Manager> {
    AccessGuardMut::new(self.ptr.write().await)
  }

  /// Gets owned immutable access to the underlying container and value `T`.
  #[inline]
  pub async fn access_owned(&self) -> OwnedAccessGuard<T, Manager> {
    OwnedAccessGuard::new(self.ptr.clone().read_owned().await)
  }

  /// Gets owned mutable access to the underlying container and value `T`.
  #[inline]
  pub async fn access_owned_mut(&self) -> OwnedAccessGuardMut<T, Manager> {
    OwnedAccessGuardMut::new(self.ptr.clone().write_owned().await)
  }

  /// Tries to get immutable access to the underlying container and value `T` without blocking.
  #[inline]
  pub fn try_access(&self) -> Option<AccessGuard<'_, T, Manager>> {
    self.ptr.try_read().map(AccessGuard::new).ok()
  }

  /// Tries to get mutable access to the underlying container and value `T` without blocking.
  #[inline]
  pub fn try_access_mut(&self) -> Option<AccessGuardMut<'_, T, Manager>> {
    self.ptr.try_write().map(AccessGuardMut::new).ok()
  }

  /// Tries to get owned immutable access to the underlying container and value `T` without blocking.
  #[inline]
  pub fn try_access_owned(&self) -> Option<OwnedAccessGuard<T, Manager>> {
    self.ptr.clone().try_read_owned().map(OwnedAccessGuard::new).ok()
  }

  /// Tries to get owned mutable access to the underlying container and value `T` without blocking.
  #[inline]
  pub fn try_access_owned_mut(&self) -> Option<OwnedAccessGuardMut<T, Manager>> {
    self.ptr.clone().try_write_owned().map(OwnedAccessGuardMut::new).ok()
  }

  /// Grants the caller immutable access to the underlying value `T`,
  /// but only for the duration of the provided function or closure.
  ///
  /// This function acquires an immutable lock on the shared state.
  pub async fn operate<F, R>(&self, operation: F) -> R
  where F: FnOnce(&T) -> R {
    operation(&*self.access().await)
  }

  /// Grants the caller mutable access to the underlying value `T`,
  /// but only for the duration of the provided function or closure.
  ///
  /// This function acquires a mutable lock on the shared state.
  pub async fn operate_mut<F, R>(&self, operation: F) -> R
  where F: FnOnce(&mut T) -> R {
    operation(&mut *self.access_mut().await)
  }
}

impl<T, Format, Lock, Mode> ContainerSharedAsync<T, FileManager<Format, Lock, Mode>>
where
  Format: FileFormat<T> + Send + 'static,
  Format::FormatError: Send + 'static,
  Lock: FileLock,
  Mode: FileMode,
  T: Send + 'static
{
  /// Opens a new [`ContainerSharedAsync`], returning an error if the file at the given path does not exist.
  pub async fn open<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error<Format::FormatError>>
  where Mode: Reading {
    let path = path.as_ref().to_owned();
    spawn_blocking!(Container::<T, _>::open(path, format)).map(From::from)
  }

  /// Opens a new [`ContainerSharedAsync`], creating a file at the given path if it does not exist, and overwriting its contents if it does.
  pub async fn create_overwrite<P: AsRef<Path>>(path: P, format: Format, value: T) -> Result<Self, Error<Format::FormatError>> {
    let path = path.as_ref().to_owned();
    spawn_blocking!(Container::<T, _>::create_overwrite(path, format, value)).map(From::from)
  }

  /// Opens a new [`ContainerSharedAsync`], writing the given value to the file if it does not exist.
  pub async fn create_or<P: AsRef<Path>>(path: P, format: Format, value: T) -> Result<Self, Error<Format::FormatError>> {
    let path = path.as_ref().to_owned();
    spawn_blocking!(Container::<T, _>::create_or(path, format, value)).map(From::from)
  }

  /// Opens a new [`ContainerSharedAsync`], writing the result of the given closure to the file if it does not exist.
  pub async fn create_or_else<P: AsRef<Path>, C>(path: P, format: Format, closure: C) -> Result<Self, Error<Format::FormatError>>
  where C: FnOnce() -> T + Send + 'static {
    let path = path.as_ref().to_owned();
    spawn_blocking!(Container::<T, _>::create_or_else(path, format, closure)).map(From::from)
  }

  /// Opens a new [`ContainerSharedAsync`], writing the default value of `T` to the file if it does not exist.
  pub async fn create_or_default<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error<Format::FormatError>>
  where T: Default {
    let path = path.as_ref().to_owned();
    spawn_blocking!(Container::<T, _>::create_or_default(path, format)).map(From::from)
  }
}

impl<T, Format, Lock, Mode> ContainerSharedAsync<T, FileManager<Format, Lock, Mode>>
where
  Format: FileFormat<T> + Send + Sync + 'static,
  Format::FormatError: Send + 'static,
  Lock: 'static,
  Mode: 'static,
  T: Send + Sync + 'static
{
  /// Grants the caller immutable access to the underlying value `T`,
  /// but only for the duration of the provided function or closure.
  /// The contents of `operation` will be treated as if they will block,
  /// and will be called through [`tokio::task::spawn_blocking`].
  pub async fn operate_nonblocking<F, R>(&self, operation: F) -> R
  where F: FnOnce(&T) -> R + Send + 'static, R: Send + 'static {
    let guard = self.access_owned().await;
    spawn_blocking!(operation(&guard))
  }

  /// Grants the caller mutable access to the underlying value `T`,
  /// but only for the duration of the provided function or closure.
  /// The contents of `operation` will be treated as if they will block,
  /// and will be called through [`tokio::task::spawn_blocking`].
  pub async fn operate_mut_nonblocking<F, R>(&self, operation: F) -> R
  where F: FnOnce(&mut T) -> R + Send + 'static, R: Send + 'static {
    let mut guard = self.access_owned_mut().await;
    spawn_blocking!(operation(&mut guard))
  }

  /// Reads a value from the managed file, replacing the current state in memory,
  /// immediately granting the caller immutable access to that state
  /// for the duration of the provided function or closure.
  ///
  /// The provided closure takes (1) a reference to the new state, and (2) the old state.
  ///
  /// This function acquires a mutable lock on the shared state.
  pub async fn operate_refresh<F, R>(&self, operation: F) -> Result<R, Error<Format::FormatError>>
  where Mode: Reading, F: FnOnce(&T, T) -> R {
    let mut guard = self.access_owned_mut().await;
    let (old_value, guard) = spawn_blocking!(guard.container_mut().refresh().map(|t| (t, guard)))?;
    let guard = OwnedAccessGuardMut::downgrade(guard);
    Ok(operation(&guard, old_value))
  }

  /// Grants the caller mutable access to the underlying value `T`,
  /// but only for the duration of the provided function or closure,
  /// immediately committing any changes made as long as no error was returned.
  ///
  /// This function acquires a mutable lock on the shared state.
  pub async fn operate_mut_commit<F, R, U>(&self, operation: F) -> Result<R, UserError<Format::FormatError, U>>
  where Mode: Writing, F: FnOnce(&mut T) -> Result<R, U> {
    let mut guard = self.access_owned_mut().await;
    let ret = operation(&mut guard).map_err(UserError::User)?;
    self.commit_guard(OwnedAccessGuardMut::downgrade(guard)).await?;
    Ok(ret)
  }

  /// Reads a value from the managed file, replacing the current state in memory.
  ///
  /// Returns the value of the previous state if the operation succeeded.
  ///
  /// This function acquires a mutable lock on the shared state.
  pub async fn refresh(&self) -> Result<T, Error<Format::FormatError>>
  where Mode: Reading {
    let mut guard = self.access_owned_mut().await;
    spawn_blocking!(guard.container_mut().refresh())
  }

  /// Writes the current in-memory state to the managed file.
  ///
  /// This function acquires an immutable lock on the shared state.
  /// Don't call this if you currently have an access guard, use [`ContainerSharedAsync::commit_guard`] instead.
  pub async fn commit(&self) -> Result<(), Error<Format::FormatError>>
  where Mode: Writing {
    let guard = self.access_owned().await;
    self.commit_guard(guard).await
  }

  /// Writes to the managed file given an access guard.
  pub async fn commit_guard(&self, guard: OwnedAccessGuard<T, FileManager<Format, Lock, Mode>>)
  -> Result<(), Error<Format::FormatError>>
  where Mode: Writing {
    spawn_blocking!(guard.container().commit())
  }

  /// Writes the given state to the managed file, replacing the in-memory state.
  pub async fn overwrite(&self, value: T) -> Result<(), Error<Format::FormatError>>
  where Mode: Writing {
    let mut guard = self.access_owned_mut().await;
    spawn_blocking!(guard.container_mut().overwrite(value))
  }
}

impl<T, Manager> Clone for ContainerSharedAsync<T, Manager> {
  #[inline]
  fn clone(&self) -> Self {
    ContainerSharedAsync { ptr: Arc::clone(&self.ptr) }
  }
}

impl<T, Manager> From<Container<T, Manager>> for ContainerSharedAsync<T, Manager> {
  #[inline]
  fn from(container: Container<T, Manager>) -> Self {
    ContainerSharedAsync { ptr: Arc::new(RwLock::new(container)) }
  }
}

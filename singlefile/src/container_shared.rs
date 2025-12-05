//! Container constructs allowing multiple-ownership managed access to a file.
//!
//! This module can be enabled with the `shared` cargo feature.

mod guards;

use crate::error::OrUserError;
use crate::container::Container;
use crate::manager::FileManager;

pub use self::guards::{
  AccessGuard,
  AccessGuardMut,
  OwnedAccessGuard,
  OwnedAccessGuardMut
};

use parking_lot::RwLock;

use std::path::Path;
use std::sync::Arc;

/// A shortcut to [`ContainerShared<T, StandardManager<Format>>`][crate::manager::standard::StandardManager].
pub type StandardContainerShared<T, Format> = ContainerShared<T, crate::manager::standard::StandardManager<Format>>;
/// A shortcut to [`StandardManagerOptions`][crate::manager::standard::StandardManagerOptions].
pub type StandardContainerSharedOptions = crate::manager::standard::StandardManagerOptions;

/// A shortcut to [`ContainerShared<T, AtomicManager<Format>>`][crate::manager::atomic::AtomicManager].
#[cfg_attr(docsrs, doc(cfg(feature = "atomic")))]
#[cfg(feature = "atomic")]
pub type AtomicContainerShared<T, Format, Support> = ContainerShared<T, crate::manager::atomic::AtomicManager<Format, Support>>;
/// A shortcut to [`AtomicManagerOptions`][crate::manager::atomic::AtomicManagerOptions].
#[cfg_attr(docsrs, doc(cfg(feature = "atomic")))]
#[cfg(feature = "atomic")]
pub type AtomicContainerSharedOptions<Support> = crate::manager::atomic::AtomicManagerOptions<Support>;

/// A container that allows synchronous atomic reference-counted, mutable access (gated by an [`RwLock`]) to the
/// underlying file and contents. Cloning this container will not clone the underlying contents, it will clone the
/// underlying pointer, allowing multiple-access.
#[repr(transparent)]
#[derive(Debug)]
pub struct ContainerShared<T, Manager> {
  ptr: Arc<RwLock<Container<T, Manager>>>
}

impl<T, Manager> ContainerShared<T, Manager> {
  /// Create a new [`ContainerShared`] from the value and manager directly.
  pub fn new(value: T, manager: Manager) -> Self {
    ContainerShared::from(Container::new(value, manager))
  }

  /// Returns the inner owned [`Container`], as long as there are no other existing pointers.
  /// Otherwise, the same [`ContainerShared`] is returned back.
  pub fn try_unwrap(self) -> Result<Container<T, Manager>, Self> {
    match Arc::try_unwrap(self.ptr) {
      Ok(inner) => Ok(RwLock::into_inner(inner)),
      Err(ptr) => Err(ContainerShared { ptr })
    }
  }

  /// Returns a mutable reference into the inner [`Container`], as long as there are no other existing pointers.
  pub fn get_mut(&mut self) -> Option<&mut Container<T, Manager>> {
    Arc::get_mut(&mut self.ptr).map(RwLock::get_mut)
  }

  /// Gets immutable access to the underlying container and value `T`.
  #[inline]
  pub fn access(&self) -> AccessGuard<'_, T, Manager> {
    AccessGuard::new(self.ptr.read())
  }

  /// Gets mutable access to the underlying container and value `T`.
  #[inline]
  pub fn access_mut(&self) -> AccessGuardMut<'_, T, Manager> {
    AccessGuardMut::new(self.ptr.write())
  }

  /// Gets owned immutable access to the underlying container and value `T`.
  #[inline]
  pub fn access_owned(&self) -> OwnedAccessGuard<T, Manager> {
    OwnedAccessGuard::new(self.ptr.read_arc())
  }

  /// Gets owned mutable access to the underlying container and value `T`.
  #[inline]
  pub fn access_owned_mut(&self) -> OwnedAccessGuardMut<T, Manager> {
    OwnedAccessGuardMut::new(self.ptr.write_arc())
  }

  /// Tries to get immutable access to the underlying container and value `T` without blocking.
  #[inline]
  pub fn try_access(&self) -> Option<AccessGuard<'_, T, Manager>> {
    self.ptr.try_read().map(AccessGuard::new)
  }

  /// Tries to get mutable access to the underlying container and value `T` without blocking.
  #[inline]
  pub fn try_access_mut(&self) -> Option<AccessGuardMut<'_, T, Manager>> {
    self.ptr.try_write().map(AccessGuardMut::new)
  }

  /// Tries to get owned immutable access to the underlying container and value `T` without blocking.
  #[inline]
  pub fn try_access_owned(&self) -> Option<OwnedAccessGuard<T, Manager>> {
    self.ptr.try_read_arc().map(OwnedAccessGuard::new)
  }

  /// Tries to get owned mutable access to the underlying container and value `T` without blocking.
  #[inline]
  pub fn try_access_owned_mut(&self) -> Option<OwnedAccessGuardMut<T, Manager>> {
    self.ptr.try_write_arc().map(OwnedAccessGuardMut::new)
  }

  /// Grants the caller immutable access to the underlying value `T`,
  /// but only for the duration of the provided function or closure.
  ///
  /// This function acquires an immutable lock on the shared state.
  pub fn operate<F, R>(&self, operation: F) -> R
  where F: FnOnce(&T) -> R {
    operation(&*self.access())
  }

  /// Grants the caller mutable access to the underlying value `T`,
  /// but only for the duration of the provided function or closure.
  ///
  /// This function acquires a mutable lock on the shared state.
  pub fn operate_mut<F, R>(&self, operation: F) -> R
  where F: FnOnce(&mut T) -> R {
    operation(&mut *self.access_mut())
  }
}

impl<T, Manager> ContainerShared<T, Manager>
where Manager: FileManager<T> {
  /// Opens a new [`ContainerShared`], returning an error if the file at the given path does not exist.
  pub fn open<P: AsRef<Path>>(
    path: P, format: Manager::Format, options: Manager::Options
  ) -> Result<Self, Manager::Error> {
    Container::<T, _>::open(path, format, options).map(From::from)
  }

  /// Opens a new [`ContainerShared`], creating a file at the given path if it does not exist, and overwriting its contents if it does.
  pub fn create_overwrite<P: AsRef<Path>>(
    path: P, format: Manager::Format, options: Manager::Options, value: T
  ) -> Result<Self, Manager::Error> {
    Container::<T, _>::create_overwrite(path, format, options, value).map(From::from)
  }

  /// Opens a new [`ContainerShared`], writing the given value to the file if it does not exist.
  pub fn create_or<P: AsRef<Path>>(
    path: P, format: Manager::Format, options: Manager::Options, value: T
  ) -> Result<Self, Manager::Error> {
    Container::<T, _>::create_or(path, format, options, value).map(From::from)
  }

  /// Opens a new [`ContainerShared`], writing the result of the given closure to the file if it does not exist.
  pub fn create_or_else<P: AsRef<Path>, C>(
    path: P, format: Manager::Format, options: Manager::Options, closure: C
  ) -> Result<Self, Manager::Error>
  where C: FnOnce() -> T {
    Container::<T, _>::create_or_else(path, format, options, closure).map(From::from)
  }

  /// Opens a new [`ContainerShared`], writing the default value of `T` to the file if it does not exist.
  pub fn create_or_default<P: AsRef<Path>>(
    path: P, format: Manager::Format, options: Manager::Options
  ) -> Result<Self, Manager::Error>
  where T: Default {
    Container::<T, _>::create_or_default(path, format, options).map(From::from)
  }

  /// Reads a value from the managed file, replacing the current state in memory,
  /// immediately granting the caller immutable access to that state
  /// for the duration of the provided function or closure.
  ///
  /// The provided closure takes (1) a reference to the new state, and (2) the old state.
  ///
  /// This function acquires a mutable lock on the shared state.
  #[doc(alias = "operate_load", alias = "operate_reload")]
  pub fn operate_refresh<F, R>(&self, operation: F) -> Result<R, Manager::Error>
  where F: FnOnce(&T, T) -> R {
    let mut guard = self.access_mut();
    let old_value = guard.container_mut().refresh()?;
    let guard = AccessGuardMut::downgrade(guard);
    Ok(operation(&guard, old_value))
  }

  /// Grants the caller mutable access to the underlying value `T`,
  /// but only for the duration of the provided function or closure,
  /// immediately committing any changes made.
  ///
  /// This function acquires a mutable lock on the shared state.
  #[doc(alias = "operate_mut_store", alias = "operate_mut_save")]
  pub fn operate_mut_commit<F, R, U>(&self, operation: F) -> Result<R, OrUserError<Manager::Error, U>>
  where F: FnOnce(&mut T) -> Result<R, U> {
    let mut guard = self.access_mut();
    let ret = operation(&mut guard).map_err(OrUserError::User)?;
    Self::commit_with_guard(guard)?;
    Ok(ret)
  }

  /// Reads a value from the managed file, replacing the current state in memory.
  ///
  /// Returns the value of the previous state if the operation succeeded.
  ///
  /// This function acquires a mutable lock on the shared state.
  #[doc(alias = "load", alias = "reload")]
  pub fn refresh(&self) -> Result<T, Manager::Error> {
    AccessGuardMut::container_mut(&mut self.access_mut()).refresh()
  }

  /// Writes the current in-memory state to the managed file.
  ///
  /// This function acquires an immutable lock on the shared state.
  /// Don't call this if you currently have an access guard,
  /// use [`ContainerShared::commit_with_guard_owned`]
  /// or [`ContainerShared::commit_with_guard`] instead.
  ///
  /// Alternatively, you may use [`ContainerShared::operate_mut_commit`].
  #[doc(alias = "store", alias = "save")]
  pub fn commit(&self) -> Result<(), Manager::Error> {
    let guard = self.access_mut();
    Self::commit_with_guard(guard)
  }

  /// Writes to the managed file given an access guard.
  pub fn commit_with_guard(mut guard: AccessGuardMut<'_, T, Manager>) -> Result<(), Manager::Error> {
    AccessGuardMut::container_mut(&mut guard).commit()
  }

  /// Writes to the managed file given an owned access guard.
  pub fn commit_with_guard_owned(mut guard: OwnedAccessGuardMut<T, Manager>) -> Result<(), Manager::Error> {
    OwnedAccessGuardMut::container_mut(&mut guard).commit()
  }

  /// Writes the given state to the managed file, replacing the in-memory state.
  #[doc(alias = "replace")]
  pub fn overwrite(&self, value: T) -> Result<(), Manager::Error> {
    AccessGuardMut::container_mut(&mut self.access_mut()).overwrite(value)
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
    ContainerShared { ptr: Arc::new(RwLock::new(container)) }
  }
}

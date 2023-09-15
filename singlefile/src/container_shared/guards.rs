use crate::container::Container;

use std::ops::{Deref, DerefMut};

type RwLockReadGuard<'a, T> = parking_lot::lock_api::RwLockReadGuard<'a, parking_lot::RawRwLock, T>;
type RwLockWriteGuard<'a, T> = parking_lot::lock_api::RwLockWriteGuard<'a, parking_lot::RawRwLock, T>;
type ArcRwLockReadGuard<T> = parking_lot::lock_api::ArcRwLockReadGuard<parking_lot::RawRwLock, T>;
type ArcRwLockWriteGuard<T> = parking_lot::lock_api::ArcRwLockWriteGuard<parking_lot::RawRwLock, T>;



#[repr(transparent)]
#[derive(Debug)]
pub struct AccessGuard<'a, T, Manager> {
  inner: RwLockReadGuard<'a, Container<T, Manager>>
}

impl<'a, T, Manager> AccessGuard<'a, T, Manager> {
  #[inline]
  pub(super) fn new(inner: RwLockReadGuard<'a, Container<T, Manager>>) -> Self {
    AccessGuard { inner }
  }

  #[inline]
  pub fn manager(&self) -> &Manager {
    Container::manager(&self.inner)
  }

  #[inline]
  pub fn container(&self) -> &Container<T, Manager> {
    &self.inner
  }
}

impl<'a, T, Manager> Deref for AccessGuard<'a, T, Manager> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    Container::get(&self.inner)
  }
}



#[repr(transparent)]
#[derive(Debug)]
pub struct AccessGuardMut<'a, T, Manager> {
  inner: RwLockWriteGuard<'a, Container<T, Manager>>
}

impl<'a, T, Manager> AccessGuardMut<'a, T, Manager> {
  #[inline]
  pub(super) fn new(inner: RwLockWriteGuard<'a, Container<T, Manager>>) -> Self {
    AccessGuardMut { inner }
  }

  #[inline]
  pub fn manager(&self) -> &Manager {
    Container::manager(&self.inner)
  }

  #[inline]
  pub fn container(&self) -> &Container<T, Manager> {
    &self.inner
  }

  #[inline]
  pub fn container_mut(&mut self) -> &mut Container<T, Manager> {
    &mut self.inner
  }

  #[inline]
  pub fn downgrade(self) -> AccessGuard<'a, T, Manager> {
    AccessGuard { inner: RwLockWriteGuard::downgrade(self.inner) }
  }
}

impl<'a, T, Manager> Deref for AccessGuardMut<'a, T, Manager> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    Container::get(&self.inner)
  }
}

impl<'a, T, Manager> DerefMut for AccessGuardMut<'a, T, Manager> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    Container::get_mut(&mut self.inner)
  }
}



#[repr(transparent)]
#[derive(Debug)]
pub struct OwnedAccessGuard<T, Manager> {
  inner: ArcRwLockReadGuard<Container<T, Manager>>
}

impl<T, Manager> OwnedAccessGuard<T, Manager> {
  #[inline]
  pub(super) fn new(inner: ArcRwLockReadGuard<Container<T, Manager>>) -> Self {
    OwnedAccessGuard { inner }
  }

  #[inline]
  pub fn manager(&self) -> &Manager {
    Container::manager(&self.inner)
  }

  #[inline]
  pub fn container(&self) -> &Container<T, Manager> {
    &self.inner
  }
}

impl<T, Manager> Deref for OwnedAccessGuard<T, Manager> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    Container::get(&self.inner)
  }
}



#[repr(transparent)]
#[derive(Debug)]
pub struct OwnedAccessGuardMut<T, Manager> {
  inner: ArcRwLockWriteGuard<Container<T, Manager>>
}

impl<T, Manager> OwnedAccessGuardMut<T, Manager> {
  #[inline]
  pub(super) fn new(inner: ArcRwLockWriteGuard<Container<T, Manager>>) -> Self {
    OwnedAccessGuardMut { inner }
  }

  #[inline]
  pub fn manager(&self) -> &Manager {
    Container::manager(&self.inner)
  }

  #[inline]
  pub fn container(&self) -> &Container<T, Manager> {
    &self.inner
  }

  #[inline]
  pub fn container_mut(&mut self) -> &mut Container<T, Manager> {
    &mut self.inner
  }

  #[inline]
  pub fn downgrade(self) -> OwnedAccessGuard<T, Manager> {
    OwnedAccessGuard { inner: ArcRwLockWriteGuard::downgrade(self.inner) }
  }
}

impl<T, Manager> Deref for OwnedAccessGuardMut<T, Manager> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    Container::get(&self.inner)
  }
}

impl<T, Manager> DerefMut for OwnedAccessGuardMut<T, Manager> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    Container::get_mut(&mut self.inner)
  }
}

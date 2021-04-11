extern crate serde;
pub extern crate serde_multi;
extern crate fs2;

mod error;
pub mod manager;

use serde_multi::traits::SerdeStream;

pub use crate::error::Error;
use crate::manager::lock::AnyLock;
use crate::manager::mode::AnyMode;
pub use crate::manager::FileManager;
pub use crate::manager::{Reading, Writing};
use crate::manager::*;

use std::path::Path;
use std::ops::{Deref, DerefMut};

pub struct Container<T, Manager> {
  item: T,
  manager: Manager
}

pub type BackendMemoryOnly<T> = Container<T, ()>;
pub type BackendReadonly<T, Format> = Container<T, ManagerReadonly<Format>>;
pub type BackendWritable<T, Format> = Container<T, ManagerWritable<Format>>;
pub type BackendReadonlyLocked<T, Format> = Container<T, ManagerReadonlyLocked<Format>>;
pub type BackendWritableLocked<T, Format> = Container<T, ManagerWritableLocked<Format>>;

impl<T, Manager> Container<T, Manager> {
  #[inline]
  pub fn into_inner(self) -> T {
    self.item
  }

  #[inline]
  pub fn into_manager(self) -> Manager {
    self.manager
  }

  #[inline]
  pub fn borrow(&self) -> &T {
    &self.item
  }

  #[inline]
  pub fn borrow_mut(&mut self) -> &mut T {
    &mut self.item
  }
}

impl<T> Container<T, ()> {
  #[inline]
  pub fn new(item: T) -> Self {
    Container { item, manager: () }
  }
}

impl<T, Format, Lock, Mode> Container<T, FileManager<Format, Lock, Mode>>
where Format: SerdeStream, Lock: AnyLock, Mode: AnyMode<Format> {
  pub fn open<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error>
  where Mode: Reading<T> {
    let manager = FileManager::open(path, format)?;
    let item = manager.read()?;
    Ok(Container { item, manager })
  }

  pub fn create_or<P: AsRef<Path>>(path: P, format: Format, item: T) -> Result<Self, Error>
  where Mode: Reading<T> + Writing<T> {
    let (item, manager) = FileManager::create_or_else(path, format, || item)?;
    Ok(Container { item, manager })
  }

  pub fn create_or_else<P: AsRef<Path>, C>(path: P, format: Format, closure: C) -> Result<Self, Error>
  where Mode: Reading<T> + Writing<T>, C: FnOnce() -> T {
    let (item, manager) = FileManager::create_or_else(path, format, closure)?;
    Ok(Container { item, manager })
  }

  pub fn create_or_default<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error>
  where T: Default, Mode: Reading<T> + Writing<T> {
    let (item, manager) = FileManager::create_or_else(path, format, || Default::default())?;
    Ok(Container { item, manager })
  }
}

impl<T, Format, Lock, Mode> Container<T, FileManager<Format, Lock, Mode>>
where Format: SerdeStream {
  pub fn refresh(&mut self) -> Result<(), Error>
  where Mode: Reading<T> {
    self.manager.read().map(|item| self.item = item)
  }

  pub fn commit(&self) -> Result<(), Error>
  where Mode: Writing<T> {
    self.manager.write(&self.item)
  }

  pub fn commit_insert(&mut self, item: T) -> Result<(), Error>
  where Mode: Writing<T> {
    self.manager.write(&item)?;
    self.item = item;
    Ok(())
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

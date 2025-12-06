//! An implementation of [`FileManager`] for general use.

use crate::error::Error;
use crate::format::FileFormat;
use crate::fs::File;
use super::{FileManager, FileLock, FileMode};

use std::io;
use std::path::Path;



/// A standard implementation of [`FileManager`]. Provides options for file locking and file modes.
#[derive(Debug)]
pub struct StandardManager<Format> {
  format: Format,
  options: StandardManagerOptions,
  file: File
}

impl<T, Format> FileManager<T> for StandardManager<Format>
where Format: FileFormat<T> {
  type Format = Format;
  type Options = StandardManagerOptions;
  type Error = Error<<Self::Format as FileFormat<T>>::FormatError>;

  fn open<P: AsRef<Path>>(
    path: P, format: Self::Format, options: Self::Options
  ) -> Result<Self, Self::Error> {
    let file = options.open(path.as_ref())?;
    Ok(StandardManager { format, options, file })
  }

  fn read(&mut self) -> Result<T, Self::Error> {
    self.options.mode.must_read()?;
    super::read(&self.format, &self.file)
  }

  fn write(&mut self, value: &T) -> Result<(), Self::Error> {
    self.options.mode.must_write()?;
    super::write(&self.format, &self.file, value)
  }

  fn into_inner(self) -> Result<(Format, Self::Options), Self::Error> {
    self.options.lock.unlock(&self.file)?;
    Ok((self.format, self.options))
  }
}

/// Options for a [`StandardManager`].
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct StandardManagerOptions {
  /// What file lock scheme the [`StandardManager`] should use.
  ///
  /// See [`FileLock`] for more information.
  pub lock: FileLock,
  /// What file mode the [`StandardManager`] should use.
  ///
  /// See [`FileMode`] for more information.
  pub mode: FileMode
}

impl StandardManagerOptions {
  /// Create a new [`StandardManagerOptions`], defaults to using [`FileLock::None`] and [`FileMode::Writable`].
  pub const fn new() -> Self {
    Self::from_lock_and_mode(FileLock::None, FileMode::Writable)
  }

  /// Create a new [`StandardManagerOptions`] given a [`FileLock`] and [`FileMode`].
  pub const fn from_lock_and_mode(lock: FileLock, mode: FileMode) -> Self {
    StandardManagerOptions { lock, mode }
  }

  /// Sets the [`FileLock`] for this [`StandardManagerOptions`].
  pub const fn with_lock(mut self, lock: FileLock) -> Self {
    self.lock = lock;
    self
  }

  /// Sets the [`FileMode`] for this [`StandardManagerOptions`].
  pub const fn with_mode(mut self, mode: FileMode) -> Self {
    self.mode = mode;
    self
  }

  /// Opens a new [`File`] with file options based on this [`StandardManagerOptions`].
  pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
    let file = self.mode.open(path.as_ref())?;
    self.lock.lock(&file)?;
    Ok(file)
  }

  /// Opens a new [`File`] with file options based on this [`StandardManagerOptions`], or create it if it does not exist.
  pub fn create<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
    let file = self.mode.create(path.as_ref())?;
    self.lock.lock(&file)?;
    Ok(file)
  }

  /// Creates a new [`File`] with file options based on this [`StandardManagerOptions`], failing if it already exists.
  pub fn create_new<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
    let file = self.mode.create_new(path.as_ref())?;
    self.lock.lock(&file)?;
    Ok(file)
  }

  /// A [`StandardManagerOptions`] set to use [`FileLock::None`] and [`FileMode::Readonly`].
  pub const UNLOCKED_READONLY: Self = StandardManagerOptions::from_lock_and_mode(FileLock::None, FileMode::Readonly);
  /// A [`StandardManagerOptions`] set to use [`FileLock::None`] and [`FileMode::Writable`].
  pub const UNLOCKED_WRITABLE: Self = StandardManagerOptions::from_lock_and_mode(FileLock::None, FileMode::Writable);
  /// A [`StandardManagerOptions`] set to use [`FileLock::Shared`] and [`FileMode::Readonly`].
  pub const LOCKED_READONLY: Self = StandardManagerOptions::from_lock_and_mode(FileLock::Shared, FileMode::Readonly);
  /// A [`StandardManagerOptions`] set to use [`FileLock::Exclusive`] and [`FileMode::Writable`].
  pub const LOCKED_WRITABLE: Self = StandardManagerOptions::from_lock_and_mode(FileLock::Exclusive, FileMode::Writable);
}

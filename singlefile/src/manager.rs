//! This module contains the [`FileManager`] struct which gives more direct access to a file.

pub mod lock;
pub mod mode;
pub mod format;

use crate::error::Error;
use crate::fs::{File, OpenOptions};
use self::lock::FileLock;
use self::mode::FileMode;
pub use self::lock::{NoLock, SharedLock, ExclusiveLock};
pub use self::mode::{Atomic, Readonly, Writable, Reading, Writing};
pub use self::format::FileFormat;

use std::io;
use std::marker::PhantomData;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::io::{IntoRawFd, AsRawFd, RawFd};
#[cfg(windows)]
use std::os::windows::io::{IntoRawHandle, AsRawHandle, RawHandle};

/// Manages a single file, allowing you to manipulate it in certain ways depending on the type parameters provided.
/// This includes file format, file locking mode, and file access mode.
#[derive(Debug)]
pub struct FileManager<Format, Lock, Mode> {
  format: Format,
  lock: PhantomData<Lock>,
  mode: PhantomData<Mode>,
  file: File
}

impl<Format, Lock, Mode> FileManager<Format, Lock, Mode>
where Lock: FileLock, Mode: FileMode {
  /// Opens a new [`FileManager`], returning an error if the file at the given path does not exist.
  pub fn open<P: AsRef<Path>>(path: P, format: Format) -> io::Result<Self> {
    let file = Mode::open(path)?;
    Lock::lock(&file)?;
    Ok(FileManager {
      format,
      lock: PhantomData,
      mode: PhantomData,
      file
    })
  }

  /// Opens a new [`FileManager`], creating a file at the given path if it does not exist, and overwriting its contents if it does.
  pub fn create_overwrite<P: AsRef<Path>, T>(path: P, format: Format, value: T) -> Result<(T, Self), Error<Format::FormatError>>
  where Format: FileFormat<T> {
    overwrite(path.as_ref(), &format, &value)?;
    Ok((value, Self::open(path, format)?))
  }

  /// Opens a new [`FileManager`], writing the given value to the file if it does not exist.
  pub fn create_or<P: AsRef<Path>, T>(path: P, format: Format, value: T) -> Result<(T, Self), Error<Format::FormatError>>
  where Format: FileFormat<T> {
    let value = read_or_write(path.as_ref(), &format, || value)?;
    Ok((value, Self::open(path, format)?))
  }

  /// Opens a new [`FileManager`], writing the result of the given closure to the file if it does not exist.
  pub fn create_or_else<P: AsRef<Path>, T, C>(path: P, format: Format, closure: C) -> Result<(T, Self), Error<Format::FormatError>>
  where Format: FileFormat<T>, C: FnOnce() -> T {
    let value = read_or_write(path.as_ref(), &format, closure)?;
    Ok((value, Self::open(path, format)?))
  }

  /// Opens a new [`FileManager`], writing the default value of `T` to the file if it does not exist.
  pub fn create_or_default<P: AsRef<Path>, T>(path: P, format: Format) -> Result<(T, Self), Error<Format::FormatError>>
  where Format: FileFormat<T>, T: Default {
    let value = read_or_write(path.as_ref(), &format, T::default)?;
    Ok((value, Self::open(path, format)?))
  }

  /// Returns a reference to the path that this file was created with.
  #[cfg_attr(docsrs, doc(cfg(any(feature = "fs-err2", feature = "fs-err3"))))]
  #[cfg(any(feature = "fs-err2", feature = "fs-err3"))]
  #[inline]
  pub fn path(&self) -> &Path {
    self.file.path()
  }
}

impl<Format, Lock, Mode> FileManager<Format, Lock, Mode>
where Lock: FileLock {
  /// Unlocks and closes this [`FileManager`].
  pub fn close(self) -> io::Result<()> {
    Lock::unlock(&self.file)?;
    self.file.sync_all()?;
    Ok(())
  }

  /// Unlocks and closes this [`FileManager`], returning the [`FileFormat`] that it uses.
  pub fn into_inner(self) -> io::Result<Format> {
    Lock::unlock(&self.file)?;
    self.file.sync_all()?;
    Ok(self.format)
  }
}

impl<Format, Lock, Mode> FileManager<Format, Lock, Mode> {
  /// Writes a given value to the file managed by this manager.
  #[inline]
  pub fn write<T>(&self, value: &T) -> Result<(), Error<Format::FormatError>>
  where Format: FileFormat<T>, Mode: Writing {
    Mode::write(&self.format, &self.file, value)
  }

  /// Reads a value from the file managed by this manager.
  #[inline]
  pub fn read<T>(&self) -> Result<T, Error<Format::FormatError>>
  where Format: FileFormat<T>, Mode: Reading {
    Mode::read(&self.format, &self.file)
  }
}

// SAFETY: `Lock` and `Mode` do not really exist within `FileManager`, they are `PhantomData`.
unsafe impl<Format: Send, Lock, Mode> Send for FileManager<Format, Lock, Mode> {}
unsafe impl<Format: Sync, Lock, Mode> Sync for FileManager<Format, Lock, Mode> {}

#[cfg(unix)]
impl<Format, Lock, Mode> IntoRawFd for FileManager<Format, Lock, Mode> {
  fn into_raw_fd(self) -> RawFd {
    self.file.into_raw_fd()
  }
}

#[cfg(unix)]
impl<Format, Lock, Mode> AsRawFd for FileManager<Format, Lock, Mode> {
  fn as_raw_fd(&self) -> RawFd {
    self.file.as_raw_fd()
  }
}

#[cfg(windows)]
impl<Format, Lock, Mode> IntoRawHandle for FileManager<Format, Lock, Mode> {
  fn into_raw_handle(self) -> RawHandle {
    self.file.into_raw_handle()
  }
}

#[cfg(windows)]
impl<Format, Lock, Mode> AsRawHandle for FileManager<Format, Lock, Mode> {
  fn as_raw_handle(&self) -> RawHandle {
    self.file.as_raw_handle()
  }
}

/// Type alias to a file manager that is read-only, and has no file lock.
pub type ManagerReadonly<Format> = FileManager<Format, NoLock, Readonly>;
/// Type alias to a file manager that is readable and writable, and has no file lock.
pub type ManagerWritable<Format> = FileManager<Format, NoLock, Writable>;
/// Type alias to a file manager that is readable and writable (with atomic writes), and has no file lock.
/// See [`Atomic`] for more information.
pub type ManagerAtomic<Format> = FileManager<Format, NoLock, Atomic>;
/// Type alias to a file manager that is read-only, and has a shared file lock.
pub type ManagerReadonlyLocked<Format> = FileManager<Format, SharedLock, Readonly>;
/// Type alias to a file manager that is readable and writable, and has an exclusive file lock.
pub type ManagerWritableLocked<Format> = FileManager<Format, ExclusiveLock, Writable>;
/// Type alias to a file manager that is readable and writable (with atomic writes), and has an exclusive file lock.
/// See [`Atomic`] for more information.
pub type ManagerAtomicLocked<Format> = FileManager<Format, ExclusiveLock, Atomic>;

fn read_or_write<T, C, Format>(path: &Path, format: &Format, closure: C) -> Result<T, Error<Format::FormatError>>
where Format: FileFormat<T>, C: FnOnce() -> T {
  use std::io::ErrorKind::NotFound;
  match OpenOptions::new().read(true).open(path) {
    Ok(file) => self::mode::read(format, &file),
    Err(err) if err.kind() == NotFound => {
      let file = OpenOptions::new()
        .write(true).create(true)
        .truncate(false)
        .open(path)?;
      let value = closure();
      self::mode::write(format, &file, &value)?;
      Ok(value)
    },
    Err(err) => Err(err.into())
  }
}

fn overwrite<T, Format>(path: &Path, format: &Format, value: &T) -> Result<(), Error<Format::FormatError>>
where Format: FileFormat<T> {
  let file = OpenOptions::new().write(true)
    .create(true).truncate(true).open(path)?;
  self::mode::write(format, &file, value)?;
  Ok(())
}

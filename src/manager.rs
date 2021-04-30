

pub mod lock;
pub mod mode;

use serde::{Serialize, Deserialize};
use serde_multi::traits::SerdeStream;

use crate::error::Error;
use self::lock::*;
use self::mode::*;

use std::path::Path;
use std::marker::PhantomData;
use std::fs::{File, OpenOptions};

/// Manages a single file, allowing you to manipulate it in certain ways depending on the type parameters provided.
#[derive(Debug)]
pub struct FileManager<Format, Lock, Mode> {
  format: PhantomData<Format>,
  lock: PhantomData<Lock>,
  mode: Mode,
  file: File
}

impl<Format, Lock, Mode> FileManager<Format, Lock, Mode>
where Format: SerdeStream, Lock: FileLock, Mode: FileMode<Format> {
  /// Open a new `FileManager`, returning an error if the file at the given path does not exist.
  pub fn open<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error> {
    let file = Mode::open(path.as_ref())?;
    Lock::lock(&file)?;
    Ok(FileManager {
      format: PhantomData,
      lock: PhantomData,
      mode: Mode::from(format),
      file
    })
  }

  /// Open a new `FileManager`, writing the given value to the file if it does not exist.
  pub fn create_or<P: AsRef<Path>, T>(path: P, format: Format, item: T) -> Result<(T, Self), Error>
  where for<'de> T: Serialize + Deserialize<'de> {
    let item = read_or_write(path.as_ref(), &format, || item)?;
    Ok((item, Self::open(path, format)?))
  }

  /// Open a new `FileManager`, writing the result of the given closure to the file if it does not exist.
  pub fn create_or_else<P: AsRef<Path>, T, C>(path: P, format: Format, closure: C) -> Result<(T, Self), Error>
  where for<'de> T: Serialize + Deserialize<'de>, C: FnOnce() -> T {
    let item = read_or_write(path.as_ref(), &format, closure)?;
    Ok((item, Self::open(path, format)?))
  }

  /// Open a new `FileManager`, writing the default value of `T` to the file if it does not exist.
  pub fn create_or_default<P: AsRef<Path>, T>(path: P, format: Format) -> Result<(T, Self), Error>
  where for<'de> T: Serialize + Deserialize<'de>, T: Default {
    let item = read_or_write(path.as_ref(), &format, T::default)?;
    Ok((item, Self::open(path, format)?))
  }
}

impl<Format, Lock, Mode> FileManager<Format, Lock, Mode>
where Lock: FileLock {
  /// Unlocks and closes this `FileManager`.
  pub fn close(self) -> Result<(), Error> {
    Lock::unlock(&self.file).map_err(From::from)
  }

  /// Unlocks and converts this `FileManager` into its underlying `File`.
  pub fn into_file(self) -> Result<File, Error> {
    Lock::unlock(&self.file)?;
    Ok(self.file)
  }
}

impl<Format, Lock, Mode> FileManager<Format, Lock, Mode>
where Format: SerdeStream {
  /// Writes a given value to the file managed by this manager.
  #[inline]
  pub fn write<T>(&self, value: &T) -> Result<(), Error>
  where Mode: Writing<T, Format> {
    self.mode.write(&self.file, value)
  }

  /// Reads a value from the file managed by this manager.
  #[inline]
  pub fn read<T>(&self) -> Result<T, Error>
  where Mode: Reading<T, Format> {
    self.mode.read(&self.file)
  }
}

/// Type alias to a read-only, unlocked `FileManager`.
pub type ManagerReadonly<Format> = FileManager<Format, NoLock, Readonly<Format>>;
/// Type alias to a readable and writable, unlocked `FileManager`.
pub type ManagerWritable<Format> = FileManager<Format, NoLock, Writable<Format>>;
/// Type alias to a read-only, shared-locked `FileManager`.
pub type ManagerReadonlyLocked<Format> = FileManager<Format, SharedLock, Readonly<Format>>;
/// Type alias to a readable and writable, exclusively-locked `FileManager`.
pub type ManagerWritableLocked<Format> = FileManager<Format, ExclusiveLock, Writable<Format>>;

#[inline]
fn read_or_write<T, C, Format>(path: &Path, format: &Format, closure: C) -> Result<T, Error>
where for<'de> T: Serialize + Deserialize<'de>, Format: SerdeStream, C: FnOnce() -> T {
  use std::io::ErrorKind::NotFound;
  match OpenOptions::new().read(true).open(path) {
    Ok(file) => self::mode::read(format, &file),
    Err(err) if err.kind() == NotFound => {
      let file = OpenOptions::new().write(true).create(true).open(path)?;
      let value = closure();
      self::mode::write(format, &file, &value)?;
      Ok(value)
    },
    Err(err) => Err(err.into())
  }
}

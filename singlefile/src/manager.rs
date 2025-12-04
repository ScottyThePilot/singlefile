//! This module contains the [`FileManager`] trait which gives more direct access to a file.

#[cfg_attr(docsrs, doc(cfg(feature = "atomic")))]
#[cfg(feature = "atomic")]
pub mod atomic;
pub mod standard;

use crate::error::{Error, OtherError};
use crate::format::FileFormat;
use crate::fs::{File, OpenOptions};

pub use self::standard::{StandardManager, StandardManagerOptions};

use std::io::{self, prelude::*, SeekFrom};
use std::path::{Path, PathBuf};

/// A [`FileManager`] and an associated [`FileFormat`] form the basis for which a file is managed.
///
/// Usually, wherever a [`FileManager`] is needed, you can use the standard implementation, [`StandardManager`].
/// You only need to manually implement [`FileManager`] if you would like to customize behavior with respect to
/// file reading/writing.
///
/// It is important to note that [`FileManager`]s are not supposed to hold any value `T` (`where Format: FileFormat<T>`)
/// that is read from and written to disk. The container types do this instead.
pub trait FileManager<T>: Sized {
  /// The [`FileFormat`] of this [`FileManager`].
  type Format: FileFormat<T>;

  /// Specifies the behavior of the [`FileManager`], required on instantiation.
  type Options;

  /// The error type returned by methods of this [`FileManager`].
  type Error: From<Error<<Self::Format as FileFormat<T>>::FormatError>>;

  /// Open a new instance of this [`FileManager`], returning an error if the file at the given path does not exist.
  fn open<P: AsRef<Path>>(
    path: P, format: Self::Format, options: Self::Options
  ) -> Result<Self, Self::Error>;

  /// Open a new instance of this [`FileManager`], creating a file at the given path if it does not exist, and overwriting its contents if it does.
  fn create_overwrite<P: AsRef<Path>>(
    path: P, format: Self::Format, options: Self::Options, value: T
  ) -> Result<(T, Self), Self::Error> {
    overwrite(path.as_ref(), &format, &value)?;
    Ok((value, Self::open(path, format, options)?))
  }

  /// Open a new instance of this [`FileManager`], writing the given value to the file if it does not exist.
  fn create_or<P: AsRef<Path>>(
    path: P, format: Self::Format, options: Self::Options, value: T
  ) -> Result<(T, Self), Self::Error> {
    Self::create_or_else(path, format, options, || value)
  }

  /// Open a new instance of this [`FileManager`], writing the result of the given closure to the file if it does not exist.
  fn create_or_else<P: AsRef<Path>>(
    path: P, format: Self::Format, options: Self::Options, closure: impl FnOnce() -> T
  ) -> Result<(T, Self), Self::Error> {
    let value = read_or_write(path.as_ref(), &format, closure)?;
    Ok((value, Self::open(path, format, options)?))
  }

  /// Open a new instance of this [`FileManager`], writing the default value of `T` to the file if it does not exist.
  fn create_or_default<P: AsRef<Path>>(
    path: P, format: Self::Format, options: Self::Options
  ) -> Result<(T, Self), Self::Error>
  where T: Default {
    Self::create_or_else(path, format, options, T::default)
  }

  /// Reads a value from the file managed by this [`FileManager`].
  fn read(&mut self) -> Result<T, Self::Error>;

  /// Writes a value to the file managed by this [`FileManager`].
  fn write(&mut self, value: &T) -> Result<(), Self::Error>;

  /// Closes this [`FileManager`], returning the `Format` and `Options` parameters used to construct it.
  fn into_inner(self) -> Result<(Self::Format, Self::Options), Self::Error>;

  /// Closes this [`FileManager`].
  fn close(self) -> Result<(), Self::Error> {
    self.into_inner().map(drop)
  }
}



fn read_or_write<T, C, Format>(path: &Path, format: &Format, closure: C) -> Result<T, Error<Format::FormatError>>
where Format: FileFormat<T>, C: FnOnce() -> T {
  use std::io::ErrorKind::NotFound;
  match OpenOptions::new().read(true).open(path) {
    Ok(file) => read(format, &file),
    Err(err) if err.kind() == NotFound => {
      let value = closure();
      overwrite(path, format, &value)?;
      Ok(value)
    },
    Err(err) => Err(err.into())
  }
}

fn overwrite<T, Format>(path: &Path, format: &Format, value: &T) -> Result<(), Error<Format::FormatError>>
where Format: FileFormat<T> {
  let file = OpenOptions::new()
    .write(true).create(true)
    .truncate(true)
    .open(path)?;
  write(format, &file, value)?;
  Ok(())
}

/// Reads a value, `T`, from a file given a [`FileFormat`].
pub fn read<T, Format>(format: &Format, mut file: &File) -> Result<T, Error<Format::FormatError>>
where Format: FileFormat<T> {
  let value = format.from_reader_buffered(file)
    .map_err(Error::Format)?;
  file.seek(SeekFrom::Start(0))?;
  Ok(value)
}

/// Writes a value, `T`, to a file given a [`FileFormat`].
pub fn write<T, Format>(format: &Format, mut file: &File, value: &T) -> Result<(), Error<Format::FormatError>>
where Format: FileFormat<T> {
  file.set_len(0)?;
  format.to_writer_buffered(file, value)
    .map_err(Error::Format)?;
  file.seek(SeekFrom::Start(0))?;
  file.sync_all()?;
  Ok(())
}



/// Describes a mode by which a file may be manipulated.
#[non_exhaustive]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum FileMode {
  /// A file mode that can only read from files.
  Readonly,
  /// A file mode that can read from and write to files.
  #[default]
  Writable
}

impl FileMode {
  /// Opens a new [`File`] with file options based on this [`FileMode`].
  pub fn open<P: Into<PathBuf> + AsRef<Path>>(self, path: P) -> io::Result<File> {
    let mut open_options = OpenOptions::new();
    open_options.read(self.can_read());
    open_options.write(self.can_write());
    open_options.open(path)
  }

  /// Returns `true` if this [`FileMode`] is allowed to read, `false` otherwise.
  pub const fn can_read(self) -> bool {
    match self {
      Self::Readonly | Self::Writable => true
    }
  }

  /// Returns `true` if this [`FileMode`] is allowed to write, `false` otherwise.
  pub const fn can_write(self) -> bool {
    match self {
      Self::Readonly => false,
      Self::Writable => true
    }
  }

  /// Throws an appropriate error if this [`FileMode`] is not allowed to read.
  pub const fn must_read(self) -> Result<(), OtherError> {
    if self.can_read() { Ok(()) } else { Err(OtherError::IncompatibleFileMode(self)) }
  }

  /// Throws an appropriate error if this [`FileMode`] is not allowed to write.
  pub const fn must_write(self) -> Result<(), OtherError> {
    if self.can_write() { Ok(()) } else { Err(OtherError::IncompatibleFileMode(self)) }
  }
}

/// Describes a mode by which a file can be locked or unlocked.
#[non_exhaustive]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum FileLock {
  /// A file lock mode that does not lock the file.
  #[default]
  None,
  /// A file lock mode that locks the file for shared access.
  Shared,
  /// A file lock mode that locks the file for exclusive access.
  Exclusive
}

impl FileLock {
  /// Locks the file.
  pub fn lock(self, file: &File) -> io::Result<()> {
    match self {
      Self::None => Ok(()),
      Self::Shared => crate::fs::FileExt::lock_shared(file),
      Self::Exclusive => crate::fs::FileExt::lock_exclusive(file)
    }
  }

  /// Unlocks the file.
  pub fn unlock(self, file: &File) -> io::Result<()> {
    match self {
      Self::None => Ok(()),
      Self::Shared => crate::fs::FileExt::unlock(file),
      Self::Exclusive => crate::fs::FileExt::unlock(file)
    }
  }
}

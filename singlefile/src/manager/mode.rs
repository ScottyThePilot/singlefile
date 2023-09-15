//! Defines different modes of accessing/manipulating files.

use crate::error::Error;
use crate::manager::format::FileFormat;

use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom};
use std::path::Path;



/// Describes a mode by which a `FileManager` can manipulate a file.
pub trait FileMode<Format>: From<Format> {
  /// Whether this file mode reads from files.
  /// If this is true, this type should implement [`Reading`], otherwise it should not.
  const READABLE: bool;
  /// Whether this file mode writes to files.
  /// If this is true, this type should implement [`Writing`], otherwise it should not.
  const WRITABLE: bool;
}

/// Extends `FileMode`, adding the ability to read from files.
pub trait Reading<T, Format>: FileMode<Format>
where Format: FileFormat<T> {
  /// Read a value from the file.
  fn read(&self, file: &File) -> Result<T, Error<Format::FormatError>>;
}

/// Extends `FileMode`, adding the ability to write to files.
pub trait Writing<T, Format>: FileMode<Format>
where Format: FileFormat<T> {
  /// Write a value to the file.
  fn write(&self, file: &File, value: &T) -> Result<(), Error<Format::FormatError>>;
}



/// A file mode that only allows reading from files.
#[derive(Debug, Clone, Default)]
pub struct Readonly<Format> {
  format: Format
}

impl<Format> From<Format> for Readonly<Format> {
  #[inline(always)]
  fn from(format: Format) -> Readonly<Format> {
    Readonly { format }
  }
}

impl<T, Format> Reading<T, Format> for Readonly<Format>
where Format: FileFormat<T> {
  #[inline]
  fn read(&self, file: &File) -> Result<T, Error<Format::FormatError>> {
    read(&self.format, file)
  }
}

impl<Format> FileMode<Format> for Readonly<Format> {
  const READABLE: bool = true;
  const WRITABLE: bool = false;
}



/// A file mode that allows reading and writing to files.
#[derive(Debug, Clone, Default)]
pub struct Writable<Format> {
  format: Format
}

impl<Format> From<Format> for Writable<Format> {
  #[inline(always)]
  fn from(format: Format) -> Writable<Format> {
    Writable { format }
  }
}

impl<T, Format> Reading<T, Format> for Writable<Format>
where Format: FileFormat<T> {
  #[inline]
  fn read(&self, file: &File) -> Result<T, Error<Format::FormatError>> {
    read(&self.format, file)
  }
}

impl<T, Format> Writing<T, Format> for Writable<Format>
where Format: FileFormat<T> {
  #[inline]
  fn write(&self, file: &File, value: &T) -> Result<(), Error<Format::FormatError>> {
    write(&self.format, file, value)
  }
}

impl<Format> FileMode<Format> for Writable<Format> {
  const READABLE: bool = true;
  const WRITABLE: bool = true;
}



/// Similar to [`Writable`], but eliminates the possibility of file corruption in the case of
/// the [`FileFormat`] failing midway during a write. The tradeoff is that file contents must be
/// buffered in memory during a write.
///
/// This does not however prevent the possibility of race-condition write corruption.
#[derive(Debug, Clone, Default)]
pub struct Atomic<Format> {
  format: Format
}

impl<Format> From<Format> for Atomic<Format> {
  #[inline(always)]
  fn from(format: Format) -> Atomic<Format> {
    Atomic { format }
  }
}

impl<T, Format> Reading<T, Format> for Atomic<Format>
where Format: FileFormat<T> {
  #[inline]
  fn read(&self, file: &File) -> Result<T, Error<Format::FormatError>> {
    read(&self.format, file)
  }
}

impl<T, Format> Writing<T, Format> for Atomic<Format>
where Format: FileFormat<T> {
  #[inline]
  fn write(&self, file: &File, value: &T) -> Result<(), Error<Format::FormatError>> {
    write_atomic(&self.format, file, value)
  }
}

impl<Format> FileMode<Format> for Atomic<Format> {
  const READABLE: bool = true;
  const WRITABLE: bool = true;
}



pub(crate) fn open<Mode, Format>(path: &Path) -> io::Result<File>
where Mode: FileMode<Format> {
  OpenOptions::new()
    .read(Mode::READABLE)
    .write(Mode::WRITABLE)
    .open(path)
}

pub(crate) fn read<T, Format>(
  format: &Format, mut file: &File
) -> Result<T, Error<Format::FormatError>>
where Format: FileFormat<T> {
  let value = format.from_reader_buffered(file)
    .map_err(Error::Format)?;
  file.seek(SeekFrom::Start(0))?;
  Ok(value)
}

pub(crate) fn write<T, Format>(
  format: &Format, mut file: &File, value: &T
) -> Result<(), Error<Format::FormatError>>
where Format: FileFormat<T> {
  file.set_len(0)?;
  format.to_writer_buffered(file, value)
    .map_err(Error::Format)?;
  file.seek(SeekFrom::Start(0))?;
  file.sync_all()?;
  Ok(())
}

pub(crate) fn write_atomic<T, Format>(
  format: &Format, mut file: &File, value: &T
) -> Result<(), Error<Format::FormatError>>
where Format: FileFormat<T> {
  file.set_len(0)?;
  let buf = format.to_buffer(value)
    .map_err(Error::Format)?;
  io::copy(&mut buf.as_slice(), &mut file)?;
  file.seek(SeekFrom::Start(0))?;
  file.sync_all()?;
  Ok(())
}

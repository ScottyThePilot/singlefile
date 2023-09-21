//! Defines different modes of accessing/manipulating files.

use crate::error::Error;
use crate::manager::format::FileFormat;
use crate::sealed::Sealed;

use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom};
use std::path::Path;



/// Describes a mode by which a `FileManager` can manipulate a file.
pub trait FileMode: Sealed + Send + Sync + 'static {
  /// Whether this file mode reads from files.
  const READABLE: bool;
  /// Whether this file mode writes to files.
  const WRITABLE: bool;

  /// Open a new file with this file mode.
  fn open<P: AsRef<Path>>(path: P) -> io::Result<File> {
    OpenOptions::new()
      .read(Self::READABLE)
      .write(Self::WRITABLE)
      .open(path)
  }
}

/// Extends `FileMode`, adding the ability to read from files.
pub trait Reading: FileMode {
  /// Read a value from the file.
  #[inline]
  fn read<T, Format>(format: &Format, file: &File) -> Result<T, Error<Format::FormatError>>
  where Format: FileFormat<T> {
    read(format, file)
  }
}

/// Extends `FileMode`, adding the ability to write to files.
pub trait Writing: FileMode {
  /// Write a value to the file.
  #[inline]
  fn write<T, Format>(format: &Format, file: &File, value: &T) -> Result<(), Error<Format::FormatError>>
  where Format: FileFormat<T> {
    write(format, file, value)
  }
}



/// A file mode that only allows reading from files.
#[derive(Debug, Clone, Copy, Default)]
pub struct Readonly;

impl Sealed for Readonly {}

impl Reading for Readonly {}

impl FileMode for Readonly {
  const READABLE: bool = true;
  const WRITABLE: bool = false;
}



/// A file mode that allows reading and writing to files.
#[derive(Debug, Clone, Copy, Default)]
pub struct Writable;

impl Sealed for Writable {}

impl Reading for Writable {}

impl Writing for Writable {}

impl FileMode for Writable {
  const READABLE: bool = true;
  const WRITABLE: bool = true;
}



/// Similar to [`Writable`], but eliminates the possibility of file corruption in the case of
/// the [`FileFormat`] failing midway during a write. The tradeoff is that file contents must be
/// buffered in memory during a write.
///
/// This does not however prevent the possibility of race-condition write corruption.
#[derive(Debug, Clone, Copy, Default)]
pub struct Atomic;

impl Sealed for Atomic {}

impl Reading for Atomic {}

impl Writing for Atomic {
  #[inline]
  fn write<T, Format>(format: &Format, file: &File, value: &T) -> Result<(), Error<Format::FormatError>>
  where Format: FileFormat<T> {
    write_atomic(format, file, value)
  }
}

impl FileMode for Atomic {
  const READABLE: bool = true;
  const WRITABLE: bool = true;
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

use serde::{Serialize, Deserialize};

use crate::error::SingleFileError;
use crate::manager::format::FileFormat;

use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom};
use std::path::Path;



/// Describes a mode by which a `FileManager` can manipulate a file.
pub trait FileMode<Format>: From<Format> {
  /// Open a file according to this mode, returning an error if it does not exist.
  fn open(path: &Path) -> io::Result<File>;
}

/// Extends `FileMode`, adding the ability to read from files.
pub trait Reading<T, Format>: FileMode<Format> {
  fn read(&self, file: &File) -> Result<T, SingleFileError>;
}

/// Extends `FileMode`, adding the ability to write to files.
pub trait Writing<T, Format>: FileMode<Format> {
  fn write(&self, file: &File, value: &T) -> Result<(), SingleFileError>;
}



/// A file mode that only allows reading from files.
#[derive(Debug, Clone)]
pub struct Readonly<Format> {
  format: Format
}

impl<Format> From<Format> for Readonly<Format> {
  #[inline(always)]
  fn from(format: Format) -> Readonly<Format> {
    Readonly { format }
  }
}

impl<Format> Default for Readonly<Format>
where Format: Default + FileFormat {
  #[inline]
  fn default() -> Self {
    Readonly { format: Format::default() }
  }
}

impl<T, Format> Reading<T, Format> for Readonly<Format>
where for<'de> T: Deserialize<'de>, Format: FileFormat {
  #[inline]
  fn read(&self, file: &File) -> Result<T, SingleFileError> {
    read(&self.format, file)
  }
}

impl<Format> FileMode<Format> for Readonly<Format> {
  #[inline]
  fn open(path: &Path) -> io::Result<File> {
    OpenOptions::new().read(true).open(path)
  }
}



/// A file mode that allows reading and writing to files.
#[derive(Debug, Clone)]
pub struct Writable<Format> {
  format: Format
}

impl<Format> From<Format> for Writable<Format> {
  #[inline(always)]
  fn from(format: Format) -> Writable<Format> {
    Writable { format }
  }
}

impl<Format> Default for Writable<Format>
where Format: Default + FileFormat {
  #[inline]
  fn default() -> Self {
    Writable { format: Format::default() }
  }
}

impl<T, Format> Reading<T, Format> for Writable<Format>
where for<'de> T: Deserialize<'de>, Format: FileFormat {
  #[inline]
  fn read(&self, file: &File) -> Result<T, SingleFileError> {
    read(&self.format, file)
  }
}

impl<T, Format> Writing<T, Format> for Writable<Format>
where T: Serialize, Format: FileFormat {
  #[inline]
  fn write(&self, file: &File, value: &T) -> Result<(), SingleFileError> {
    write(&self.format, file, value)
  }
}

impl<Format> FileMode<Format> for Writable<Format> {
  #[inline]
  fn open(path: &Path) -> io::Result<File> {
    OpenOptions::new().read(true).write(true).open(path)
  }
}



pub(crate) fn read<T, Format>(format: &Format, mut file: &File) -> Result<T, SingleFileError>
where for<'de> T: Deserialize<'de>, Format: FileFormat {
  let item = format.from_reader(file)
    .map_err(SingleFileError::Format)?;
  file.seek(SeekFrom::Start(0))?;
  Ok(item)
}

pub(crate) fn write<T, Format>(format: &Format, mut file: &File, value: &T) -> Result<(), SingleFileError>
where T: Serialize, Format: FileFormat {
  file.set_len(0)?;
  format.to_writer(file, value)
    .map_err(SingleFileError::Format)?;
  file.seek(SeekFrom::Start(0))?;
  file.sync_all()?;
  Ok(())
}

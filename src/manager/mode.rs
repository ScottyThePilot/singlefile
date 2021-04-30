use serde::{Serialize, Deserialize};
use serde_multi::traits::SerdeStream;

use crate::error::Error;

use std::fs::{File, OpenOptions};
use std::path::Path;
use std::io;



/// Describes a mode by which a `FileManager` can manipulate a file.
pub trait FileMode<Format>: From<Format> {
  /// Open a file according to this mode, returning an error if it does not exist.
  fn open(path: &Path) -> io::Result<File>;
}

/// Extends `FileMode`, adding the ability to read from files.
pub trait Reading<T, Format>: FileMode<Format> {
  fn read(&self, file: &File) -> Result<T, Error>;
}

/// Extends `FileMode`, adding the ability to write to files.
pub trait Writing<T, Format>: FileMode<Format> {
  fn write(&self, file: &File, value: &T) -> Result<(), Error>;
}



/// A file mode that only allows reading from files.
#[derive(Debug, Clone)]
pub struct Readonly<Format> {
  format: Format
}

impl<Format> From<Format> for Readonly<Format> {
  #[inline]
  fn from(format: Format) -> Readonly<Format> {
    Readonly { format }
  }
}

impl<Format> Default for Readonly<Format>
where Format: Default + SerdeStream {
  #[inline]
  fn default() -> Self {
    Readonly { format: Format::default() }
  }
}

impl<T, Format> Reading<T, Format> for Readonly<Format>
where for<'de> T: Deserialize<'de>, Format: SerdeStream {
  #[inline]
  fn read(&self, file: &File) -> Result<T, Error> {
    self.format.from_reader(file).map_err(From::from)
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
  #[inline]
  fn from(format: Format) -> Writable<Format> {
    Writable { format }
  }
}

impl<Format> Default for Writable<Format>
where Format: Default + SerdeStream {
  #[inline]
  fn default() -> Self {
    Writable { format: Format::default() }
  }
}

impl<T, Format> Reading<T, Format> for Writable<Format>
where for<'de> T: Deserialize<'de>, Format: SerdeStream {
  #[inline]
  fn read(&self, file: &File) -> Result<T, Error> {
    read(&self.format, file)
  }
}

impl<T, Format> Writing<T, Format> for Writable<Format>
where T: Serialize, Format: SerdeStream {
  #[inline]
  fn write(&self, file: &File, value: &T) -> Result<(), Error> {
    write(&self.format, file, value)
  }
}

impl<Format> FileMode<Format> for Writable<Format> {
  #[inline]
  fn open(path: &Path) -> io::Result<File> {
    OpenOptions::new().read(true).write(true).open(path)
  }
}



#[inline]
pub(crate) fn read<T, Format>(format: &Format, file: &File) -> Result<T, Error>
where for<'de> T: Deserialize<'de>, Format: SerdeStream {
  format.from_reader(file).map_err(From::from)
}

#[inline]
pub(crate) fn write<T, Format>(format: &Format, file: &File, value: &T) -> Result<(), Error>
where T: Serialize, Format: SerdeStream {
  format.to_writer_pretty(file, value)?;
  file.sync_all()?;
  Ok(())
}

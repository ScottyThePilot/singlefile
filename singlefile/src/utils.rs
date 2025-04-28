//! Utility functions.

use crate::error::Error;
use crate::fs::File;
use crate::manager::format::FileFormat;

use std::path::Path;

/// Helper function that reads a value with the given [`FileFormat`] from the given path.
pub fn read<T, Format>(path: impl AsRef<Path>, format: Format) -> Result<T, Error<Format::FormatError>>
where Format: FileFormat<T> {
  let file = File::open(path.as_ref())?;
  let value = format.from_reader_buffered(file).map_err(Error::Format)?;
  Ok(value)
}

/// Helper function that writes a value with the given [`FileFormat`] from the given path.
pub fn write<T, Format>(path: impl AsRef<Path>, format: Format, value: &T) -> Result<(), Error<Format::FormatError>>
where Format: FileFormat<T> {
  let file = File::create(path.as_ref())?;
  format.to_writer_buffered(file, value).map_err(Error::Format)?;
  Ok(())
}

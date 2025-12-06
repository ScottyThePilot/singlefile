//! An implementation of [`FileManager`] capable of atomic writes, which may mitigate file corruption.

use crate::error::Error;
use crate::format::FileFormat;
use crate::fs::{File, OpenOptions};
use super::{FileManager, FileLock};

use std::io;
use std::path::{Path, PathBuf};



/// An implementation of [`FileManager`]. Provides options for file locking and atomic file writes.
///
/// To use a [`AtomicManager`], you will need to provide a type implementing [`AtomicFileSupport`]
/// in the [`AtomicManagerOptions`]. [`AtomicFileSupport`] determines the scheme for where
/// temporary files will be placed and what they will be named.
#[derive(Debug)]
pub struct AtomicManager<Format, Support> {
  format: Format,
  options: AtomicManagerOptions<Support>,
  file: File
}

impl<T, Format, Support> FileManager<T> for AtomicManager<Format, Support>
where Format: FileFormat<T>, Support: AtomicFileSupport {
  type Format = Format;
  type Options = AtomicManagerOptions<Support>;
  type Error = Error<<Self::Format as FileFormat<T>>::FormatError>;

  fn open<P: AsRef<Path>>(
    path: P, format: Self::Format, options: Self::Options
  ) -> Result<Self, Self::Error> {
    let file = options.open(path.as_ref())?;
    Ok(AtomicManager { format, options, file })
  }

  fn read(&mut self) -> Result<T, Self::Error> {
    super::read(&self.format, &self.file)
  }

  fn write(&mut self, value: &T) -> Result<(), Self::Error> {
    self.options.write_atomic(&self.format, &mut self.file, value)
  }

  fn into_inner(self) -> Result<(Self::Format, Self::Options), Self::Error> {
    self.options.lock.unlock(&self.file)?;
    Ok((self.format, self.options))
  }
}

/// Options for a [`AtomicManager`].
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct AtomicManagerOptions<Support> {
  /// What file lock scheme the [`AtomicManager`] should use.
  ///
  /// See [`FileLock`] for more information.
  pub lock: FileLock,
  /// What supporting logic the [`AtomicManager`] should use.
  ///
  /// See [`AtomicFileSupport`] for more information.
  pub support: Support
}

impl<Support> AtomicManagerOptions<Support> where Support: AtomicFileSupport {
  /// Create a new [`AtomicManagerOptions`] given an [`AtomicFileSupport`], defaults to using [`FileLock::None`].
  pub const fn new(support: Support) -> Self {
    AtomicManagerOptions { lock: FileLock::None, support }
  }

  /// Sets the [`FileLock`] for this [`AtomicManagerOptions`].
  pub const fn with_lock(mut self, lock: FileLock) -> Self {
    self.lock = lock;
    self
  }

  /// Opens a new [`File`] with file options based on this [`AtomicManagerOptions`].
  pub fn open<P: Into<PathBuf>>(&self, path: P) -> io::Result<File> {
    let file = OpenOptions::new()
      .read(true).write(true)
      .open(path)?;
    self.lock.lock(&file)?;
    Ok(file)
  }

  /// Opens a new [`File`] with file options based on this [`AtomicManagerOptions`], or create it if it does not exist.
  pub fn create<P: Into<PathBuf>>(&self, path: P) -> io::Result<File> {
    let file = OpenOptions::new()
      .read(true).write(true)
      .create(true).truncate(false)
      .open(path)?;
    self.lock.lock(&file)?;
    Ok(file)
  }

  /// Creates a new [`File`] with file options based on this [`AtomicManagerOptions`], failing if it already exists.
  pub fn create_new<P: Into<PathBuf>>(&self, path: P) -> io::Result<File> {
    let file = OpenOptions::new()
      .read(true).write(true)
      .create_new(true)
      .open(path)?;
    self.lock.lock(&file)?;
    Ok(file)
  }

  /// Writes a value, `T`, to a file given a [`FileFormat`], in an atomic manner.
  pub fn write_atomic<T, Format>(&mut self, format: &Format, file: &mut File, value: &T) -> Result<(), Error<Format::FormatError>>
  where Format: FileFormat<T> {
    let new_file_path = self.support.pick_temporary_file_location(file.path())?;
    let new_file = self.create_new(new_file_path)?;
    format.to_writer_buffered(&new_file, value)
      .map_err(Error::Format)?;
    new_file.sync_all()?;
    replace_file(file, new_file)?;
    Ok(())
  }
}

/// Supporting logic for atomic file writes.
pub trait AtomicFileSupport {
  /// Picks a file path, at which a new file should be created.
  ///
  /// When an atomic write is performed, this file will be written to, and swapped in to replace the previous file.
  ///
  /// The returned file path must:
  /// - Never conflict with an existing file or directory.
  /// - Have all of its parent directories populated.
  fn pick_temporary_file_location(&mut self, current: &Path) -> io::Result<PathBuf>;
}



/// Replaces `file_dest` with `file_src`, both in memory and on disk.
fn replace_file(file_dest: &mut File, file_src: File) -> io::Result<()> {
  crate::fs::rename(file_src.path(), file_dest.path())?;
  *file_dest.file_mut() = file_src.into_file();

  Ok(())
}

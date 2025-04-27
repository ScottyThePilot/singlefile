//! Defines different types of file system locks.

use crate::sealed::Sealed;

use crate::fs::File;
use std::io;



/// Describes a mode by which a file can be locked or unlocked.
pub trait FileLock: Sealed + Send + Sync + 'static {
  /// Locks the file.
  #[doc(hidden)]
  fn lock(file: &File) -> io::Result<()>;

  /// Unlocks the file.
  #[doc(hidden)]
  fn unlock(file: &File) -> io::Result<()>;
}



/// A file lock mode that does not lock the file.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoLock;

impl Sealed for NoLock {}

impl FileLock for NoLock {
  #[inline(always)]
  fn lock(_: &File) -> io::Result<()> {
    Ok(())
  }

  #[inline(always)]
  fn unlock(_: &File) -> io::Result<()> {
    Ok(())
  }
}



/// A file lock mode that locks the file for shared access.
#[derive(Debug, Default, Clone, Copy)]
pub struct SharedLock;

impl Sealed for SharedLock {}

impl FileLock for SharedLock {
  #[inline(always)]
  fn lock(file: &File) -> io::Result<()> {
    crate::fs::FileExt::try_lock_shared(file)
  }

  #[inline(always)]
  fn unlock(file: &File) -> io::Result<()> {
    crate::fs::FileExt::unlock(file)
  }
}



/// A file lock mode that locks the file for exclusive access.
#[derive(Debug, Default, Clone, Copy)]
pub struct ExclusiveLock;

impl Sealed for ExclusiveLock {}

impl FileLock for ExclusiveLock {
  #[inline(always)]
  fn lock(file: &File) -> io::Result<()> {
    crate::fs::FileExt::try_lock_exclusive(file)
  }

  #[inline(always)]
  fn unlock(file: &File) -> io::Result<()> {
    crate::fs::FileExt::unlock(file)
  }
}

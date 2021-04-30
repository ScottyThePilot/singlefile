use std::fs::File;
use std::io;



/// Describes a mode by which a file can be locked or unlocked.
pub trait FileLock {
  fn lock(file: &File) -> io::Result<()>;

  fn unlock(file: &File) -> io::Result<()>;
}



/// A file lock mode that does not lock the file.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoLock;

impl FileLock for NoLock {
  #[inline]
  fn lock(_: &File) -> io::Result<()> {
    Ok(())
  }

  #[inline]
  fn unlock(_: &File) -> io::Result<()> {
    Ok(())
  }
}



/// A file lock mode that locks the file for shared access.
#[derive(Debug, Default, Clone, Copy)]
pub struct SharedLock;

impl FileLock for SharedLock {
  #[inline]
  fn lock(file: &File) -> io::Result<()> {
    fs2::FileExt::try_lock_shared(file)
  }

  #[inline]
  fn unlock(file: &File) -> io::Result<()> {
    fs2::FileExt::unlock(file)
  }
}



/// A file lock mode that locks the file for exclusive access.
#[derive(Debug, Default, Clone, Copy)]
pub struct ExclusiveLock;

impl FileLock for ExclusiveLock {
  #[inline]
  fn lock(file: &File) -> io::Result<()> {
    fs2::FileExt::try_lock_exclusive(file)
  }

  #[inline]
  fn unlock(file: &File) -> io::Result<()> {
    fs2::FileExt::unlock(file)
  }
}

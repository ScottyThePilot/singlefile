use std::path::{PathBuf, Path};
use std::fs::File;
use std::io;

pub trait AnyLock {
  fn new<P: AsRef<Path>>(path: P) -> io::Result<Self>
  where Self: Sized;
}

pub struct NoLock;

impl NoLock {
  pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
    File::open(path).map(|_| NoLock)
  }
}

impl AnyLock for NoLock {
  #[inline]
  fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
    NoLock::new(path)
  }
}

pub struct SharedLock {
  path: PathBuf
}

impl SharedLock {
  pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
    let file = File::open(&path)?;
    fs2::FileExt::try_lock_shared(&file)?;
    let path = path.as_ref().to_owned();
    Ok(SharedLock { path })
  }
}

impl AnyLock for SharedLock {
  fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
    SharedLock::new(path)
  }
}

impl Drop for SharedLock {
  #[inline]
  fn drop(&mut self) {
    if let Ok(file) = File::open(&self.path) {
      #[cfg(feature = "assert-file-unlocks")]
      fs2::FileExt::unlock(&file).unwrap();
      #[cfg(not(feature = "assert-file-unlocks"))]
      let _ = fs2::FileExt::unlock(&file);
    };
  }
}

pub struct ExclusiveLock {
  path: PathBuf
}

impl ExclusiveLock {
  pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
    let file = File::open(&path)?;
    fs2::FileExt::try_lock_exclusive(&file)?;
    let path = path.as_ref().to_owned();
    Ok(ExclusiveLock { path })
  }
}

impl AnyLock for ExclusiveLock {
  #[inline]
  fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
    ExclusiveLock::new(path)
  }
}

impl Drop for ExclusiveLock {
  #[inline]
  fn drop(&mut self) {
    if let Ok(file) = File::open(&self.path) {
      #[cfg(feature = "assert-file-unlocks")]
      fs2::FileExt::unlock(&file).unwrap();
      #[cfg(not(feature = "assert-file-unlocks"))]
      let _ = fs2::FileExt::unlock(&file);
    };
  }
}

pub mod lock;
pub mod mode;

use serde_multi::traits::SerdeStream;

use crate::error::Error;
use self::lock::*;
use self::mode::*;

use std::path::{Path, PathBuf};
use std::fs::File;
use std::marker::PhantomData;

pub use self::lock::NoLock;
pub use self::lock::SharedLock;
pub use self::lock::ExclusiveLock;
pub use self::mode::Readonly;
pub use self::mode::Writable;
pub use self::mode::Reading;
pub use self::mode::Writing;

pub struct FileManager<Format, Lock, Mode> {
  path: PathBuf,
  format: PhantomData<Format>,
  #[allow(dead_code)]
  lock: Lock,
  mode: Mode
}

impl<Format, Lock, Mode> FileManager<Format, Lock, Mode>
where Format: SerdeStream, Lock: AnyLock, Mode: AnyMode<Format> {
  pub fn open<P: AsRef<Path>>(path: P, format: Format) -> Result<Self, Error> {
    Ok(FileManager {
      path: path.as_ref().to_owned(),
      format: PhantomData,
      lock: Lock::new(path)?,
      mode: Mode::new(format)
    })
  }

  pub(crate) fn create_or_else<P: AsRef<Path>, T, C>(path: P, format: Format, closure: C) -> Result<(T, Self), Error>
  where Mode: Reading<T> + Writing<T>, C: FnOnce() -> T {
    let path = path.as_ref().to_owned();
    let mode = Mode::new(format);
    let item = read_or_write(&path, &mode, closure)?;
    let lock = Lock::new(&path)?;
    Ok((item, FileManager { path, format: PhantomData, mode, lock }))
  }
}

impl<Format, Lock, Mode> FileManager<Format, Lock, Mode>
where Format: SerdeStream {
  pub fn write<T>(&self, value: &T) -> Result<(), Error>
  where Mode: Writing<T> {
    let file = File::create(&self.path)?;
    self.mode.write(&file, value)
  }

  pub fn read<T>(&self) -> Result<T, Error>
  where Mode: Reading<T> {
    let file = File::create(&self.path)?;
    self.mode.read(&file)
  }

  /*pub fn read_or_write<T, C>(&self, closure: C) -> Result<T, Error>
  where Mode: Reading<T> + Writing<T>, C: FnOnce() -> T {
    let file = File::create(&self.path)?;
    match self.mode.read(&file) {
      Ok(item) => Ok(item),
      Err(err) if err.is_not_found() => {
        let item = closure();
        self.mode.write(&file, &item)?;
        file.sync_all()?;
        Ok(item)
      },
      Err(err) => Err(err)
    }
  }*/
}

pub type ManagerReadonly<Format> = FileManager<Format, NoLock, Readonly<Format>>;
pub type ManagerWritable<Format> = FileManager<Format, NoLock, Writable<Format>>;
pub type ManagerReadonlyLocked<Format> = FileManager<Format, SharedLock, Readonly<Format>>;
pub type ManagerWritableLocked<Format> = FileManager<Format, ExclusiveLock, Writable<Format>>;

fn read_or_write<Mode, T, C>(path: &Path, mode: &Mode, closure: C) -> Result<T, Error>
where Mode: Writing<T> + Reading<T>, C: FnOnce() -> T {
  use std::io::ErrorKind::NotFound;
  match File::open(&path) {
    Ok(file) => mode.read(&file),
    Err(err) if err.kind() == NotFound => {
      let file = File::create(&path)?;
      let item = closure();
      mode.write(&file, &item)?;
      Ok(item)
    },
    Err(err) => Err(err.into())
  }
}

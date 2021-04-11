use serde::{Serialize, Deserialize};
use serde_multi::traits::SerdeStream;

use crate::error::Error;

use std::fs::File;

pub trait AnyMode<Format> {
  fn new(format: Format) -> Self where Self: Sized;
}

pub trait Reading<T> {
  fn read(&self, file: &File) -> Result<T, Error>;
}

pub trait Writing<T> {
  fn write(&self, file: &File, value: &T) -> Result<(), Error>;
}

pub struct Readonly<Format> {
  format: Format
}

impl<Format> Readonly<Format>
where Format: SerdeStream {
  #[inline]
  fn new(format: Format) -> Self {
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

impl<T, Format> Reading<T> for Readonly<Format>
where for<'de> T: Deserialize<'de>, Format: SerdeStream {
  #[inline]
  fn read(&self, file: &File) -> Result<T, Error> {
    self.format.from_reader(file).map_err(From::from)
  }
}

impl<Format> AnyMode<Format> for Readonly<Format>
where Format: SerdeStream {
  #[inline]
  fn new(format: Format) -> Self {
    Readonly::new(format)
  }
}

pub struct Writable<Format> {
  format: Format
}

impl<Format> Writable<Format>
where Format: SerdeStream {
  #[inline]
  fn new(format: Format) -> Self {
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

impl<T, Format> Reading<T> for Writable<Format>
where for<'de> T: Deserialize<'de>, Format: SerdeStream {
  #[inline]
  fn read(&self, file: &File) -> Result<T, Error> {
    self.format.from_reader(file).map_err(From::from)
  }
}

impl<T, Format> Writing<T> for Writable<Format>
where T: Serialize, Format: SerdeStream {
  #[inline]
  fn write(&self, file: &File, value: &T) -> Result<(), Error> {
    self.format.to_writer_pretty(file, value)?;
    file.sync_all()?;
    Ok(())
  }
}

impl<Format> AnyMode<Format> for Writable<Format>
where Format: SerdeStream {
  #[inline]
  fn new(format: Format) -> Self {
    Writable::new(format)
  }
}

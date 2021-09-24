use std::fmt;

pub use crate::manager::format::FormatError;

#[derive(Debug)]
pub enum Error {
  /// An error caused by an implementation of [`FileFormat`].
  ///
  /// [`FileFormat`]: crate::manager::format::FileFormat
  Format(FormatError),
  /// An error caused by the filesystem.
  Io(std::io::Error)
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Error::Format(err) => Some(&**err),
      Error::Io(err) => err.source()
    }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::Format(err) => write!(f, "format error: {}", err),
      Error::Io(err) => write!(f, "{}", err)
    }
  }
}

impl From<FormatError> for Error {
  #[inline]
  fn from(source: FormatError) -> Self {
    Error::Format(source)
  }
}

impl From<std::io::Error> for Error {
  #[inline]
  fn from(source: std::io::Error) -> Self {
    Error::Io(source)
  }
}

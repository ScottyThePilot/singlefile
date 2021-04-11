use std::fmt;

#[derive(Debug)]
pub enum Error {
  Format(serde_multi::Error),
  Io(std::io::Error)
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::Format(error) => write!(f, "{}", error),
      Error::Io(error) => write!(f, "{}", error)
    }
  }
}

impl From<serde_multi::Error> for Error {
  fn from(error: serde_multi::Error) -> Error {
    Error::Format(error)
  }
}

impl From<std::io::Error> for Error {
  fn from(error: std::io::Error) -> Error {
    Error::Io(error)
  }
}

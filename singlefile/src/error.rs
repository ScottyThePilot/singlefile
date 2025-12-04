//! Errors that can occur within `singlefile`.

use thiserror::Error;

use crate::manager::FileMode;

use std::convert::Infallible;
use std::io;

/// An error that can occur within `singlefile`.
#[derive(Debug, Error)]
pub enum Error<FE> {
  /// An error caused by an implementation of [`FileFormat`].
  ///
  /// [`FileFormat`]: crate::format::FileFormat
  #[error("format error: {0}")]
  Format(FE),
  /// An error caused by the filesystem.
  #[error(transparent)]
  Io(#[from] io::Error),
  /// Any other kind of error.
  #[error(transparent)]
  Other(#[from] OtherError)
}

impl From<Error<io::Error>> for io::Error {
  fn from(err: Error<io::Error>) -> Self {
    match err {
      Error::Format(err) | Error::Io(err) => err,
      Error::Other(err) => io::Error::other(err)
    }
  }
}

/// A value that may be an error or a user-generated error.
#[derive(Debug, Error)]
pub enum OrUserError<T, U> {
  /// An error.
  #[error(transparent)]
  Base(T),
  /// A user-generated error.
  #[error("user error: {0}")]
  User(U)
}

impl<T, U> OrUserError<T, U> {
  /// Converts this error into another error.
  /// The new error type `E` must implement [`From<Error<FE>>`][enum@Error].
  /// Additionally takes a closure allowing the user to manually convert the user error.
  pub fn convert_into<E, F>(self, f: F) -> E
  where T: Into<E>, F: FnOnce(U) -> E {
    match self {
      Self::Base(err) => err.into(),
      Self::User(err) => f(err)
    }
  }
}

impl<T> OrUserError<T, Infallible> {
  /// Converts this error to `T`, given that `U` is [`Infallible`].
  pub fn into_base(self) -> T {
    match self {
      Self::Base(err) => err,
      Self::User(i) => match i {}
    }
  }
}

impl<T, U> From<T> for OrUserError<T, U> {
  fn from(err: T) -> Self {
    OrUserError::Base(err)
  }
}

/// An error that can occur within `singlefile`, with the exception of:
/// - Format Errors
/// - I/O Errors
/// - User Errors
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum OtherError {
  /// The user tried to perform an operation that is incompatible with the set [`FileMode`].
  #[error("file mode {0:?} is incompatible with this operation")]
  IncompatibleFileMode(FileMode)
}

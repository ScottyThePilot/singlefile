//! Errors that can occur within `singlefile`.

use thiserror::Error;

use std::convert::Infallible;
use std::io;

/// An error that can occur within `singlefile`.
#[derive(Debug, Error)]
pub enum Error<FE> {
  /// An error caused by an implementation of [`FileFormat`].
  ///
  /// [`FileFormat`]: crate::manager::format::FileFormat
  #[error("format error: {0}")]
  Format(FE),
  /// An error caused by the filesystem.
  #[error(transparent)]
  Io(#[from] io::Error)
}

impl<FE> From<UserError<FE, Infallible>> for Error<FE> {
  fn from(err: UserError<FE, Infallible>) -> Self {
    match err {
      UserError::Format(err) => Error::Format(err),
      UserError::Io(err) => Error::Io(err),
      UserError::User(i) => match i {}
    }
  }
}

impl From<Error<io::Error>> for io::Error {
  fn from(err: Error<io::Error>) -> Self {
    match err {
      Error::Format(err) | Error::Io(err) => err
    }
  }
}

/// An error that can occur within `singlefile`, or an error from a user operation.
#[derive(Debug, Error)]
pub enum UserError<FE, U> {
  /// An error caused by an implementation of [`FileFormat`].
  ///
  /// [`FileFormat`]: crate::manager::format::FileFormat
  #[error("format error: {0}")]
  Format(FE),
  /// An error caused by the filesystem.
  #[error(transparent)]
  Io(#[from] std::io::Error),
  /// An error caused by the user.
  #[error("user error: {0}")]
  User(U)
}

impl<FE, U> UserError<FE, U> {
  /// Maps this error into another error.
  /// The new error type `E` must implement [`From<Error<FE>>`][enum@Error].
  /// Additionally takes a closure allowing the user to manually convert the user error.
  pub fn map_into<E, F>(self, f: F) -> E
  where Error<FE>: Into<E>, F: FnOnce(U) -> E {
    match self {
      UserError::Format(err) => Error::Format(err).into(),
      UserError::Io(err) => Error::Io(err).into(),
      UserError::User(err) => f(err)
    }
  }
}

/// Converts an [`enum@Error<io::Error>`] into just an [`io::Error`].
impl<FE, U> From<Error<FE>> for UserError<FE, U> {
  fn from(err: Error<FE>) -> Self {
    match err {
      Error::Format(err) => UserError::Format(err),
      Error::Io(err) => UserError::Io(err)
    }
  }
}

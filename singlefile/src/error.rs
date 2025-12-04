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

#[allow(deprecated)]
impl<FE> From<UserError<FE, Infallible>> for Error<FE> {
  fn from(err: UserError<FE, Infallible>) -> Self {
    match err {
      UserError::Format(err) => Error::Format(err),
      UserError::Io(err) => Error::Io(err),
      UserError::Other(err) => Error::Other(err),
      UserError::User(i) => match i {}
    }
  }
}

impl From<Error<io::Error>> for io::Error {
  fn from(err: Error<io::Error>) -> Self {
    match err {
      Error::Format(err) | Error::Io(err) => err,
      Error::Other(err) => io::Error::new(io::ErrorKind::Other, err)
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

/// An error that can occur within `singlefile`, or an error from a user operation.
#[deprecated]
#[derive(Debug, Error)]
pub enum UserError<FE, U> {
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
  Other(#[from] OtherError),
  /// An error caused by the user.
  #[error("user error: {0}")]
  User(U)
}

#[allow(deprecated)]
impl<FE, U> UserError<FE, U> {
  /// Maps this error into another error.
  /// The new error type `E` must implement [`From<Error<FE>>`][enum@Error].
  /// Additionally takes a closure allowing the user to manually convert the user error.
  pub fn map_into<E, F>(self, f: F) -> E
  where Error<FE>: Into<E>, F: FnOnce(U) -> E {
    match self {
      UserError::Format(err) => Error::Format(err).into(),
      UserError::Io(err) => Error::Io(err).into(),
      UserError::Other(err) => Error::Other(err).into(),
      UserError::User(err) => f(err)
    }
  }
}

#[allow(deprecated)]
impl<FE, U> From<Error<FE>> for UserError<FE, U> {
  fn from(err: Error<FE>) -> Self {
    match err {
      Error::Format(err) => UserError::Format(err),
      Error::Io(err) => UserError::Io(err),
      Error::Other(err) => UserError::Other(err)
    }
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

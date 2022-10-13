use thiserror::Error;

pub use crate::manager::format::FormatError;

pub type SingleFileResult<T = ()> = Result<T, SingleFileError>;
pub type SingleFileUserResult<T, U> = Result<T, SingleFileUserError<U>>;

#[derive(Debug, Error)]
pub enum SingleFileError {
  /// An error caused by an implementation of [`FileFormat`].
  ///
  /// [`FileFormat`]: crate::manager::format::FileFormat
  #[error("format error: {0}")]
  Format(FormatError),
  /// An error caused by the filesystem.
  #[error(transparent)]
  Io(#[from] std::io::Error)
}

#[derive(Debug, Error)]
pub enum SingleFileUserError<U> {
  /// An error caused by an implementation of [`FileFormat`].
  ///
  /// [`FileFormat`]: crate::manager::format::FileFormat
  #[error("format error: {0}")]
  Format(FormatError),
  /// An error caused by the filesystem.
  #[error(transparent)]
  Io(#[from] std::io::Error),
  /// An error caused by the user.
  #[error("user error: {0}")]
  User(U)
}

impl<U> SingleFileUserError<U> {
  /// Maps this error into another error.
  /// The new error type `E` must implement [`From<SingleFileError>`][SingleFileError].
  /// Additionally takes a closure allowing the user to manually convert the user error.
  pub fn map_into<E, F>(self, f: F) -> E
  where SingleFileError: Into<E>, F: FnOnce(U) -> E {
    match self {
      SingleFileUserError::Format(err) => SingleFileError::Format(err).into(),
      SingleFileUserError::Io(err) => SingleFileError::Io(err).into(),
      SingleFileUserError::User(err) => f(err)
    }
  }
}

impl<U> From<SingleFileError> for SingleFileUserError<U> {
  fn from(err: SingleFileError) -> Self {
    match err {
      SingleFileError::Format(err) => SingleFileUserError::Format(err),
      SingleFileError::Io(err) => SingleFileUserError::Io(err)
    }
  }
}

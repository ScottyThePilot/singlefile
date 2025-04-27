#![allow(unused_imports)]
//! Utility module re-exporting filesystem functions.
//!
//! If the `fs-err` feature is enabled, all of these functions will

use std::path::{Path, PathBuf};
use std::io::{self, prelude::*};

#[doc(no_inline)]
pub use fs4::{
  FsStats as Stats,
  statvfs as stats,
  lock_contended_error
};

#[cfg(not(feature = "fs-err"))]
#[doc(no_inline)]
pub use fs4::fs_std::FileExt;

#[cfg(feature = "fs-err")]
#[doc(no_inline)]
pub use fs4::fs_err::FileExt;

#[doc(no_inline)]
pub use std::fs::{
  FileTimes,
  FileType,
  Metadata,
  Permissions
};

macro_rules! switch_fs {
  ($vis:vis use { $($name:ident $(as $new_name:ident)?),* $(,)? }) => (
    #[cfg(not(feature = "fs-err"))]
    #[doc(no_inline)]
    $vis use std::fs::{ $($name $(as $new_name)?),* };
    #[cfg(feature = "fs-err")]
    #[doc(no_inline)]
    $vis use fs_err::{ $($name $(as $new_name)?),* };
  );
}

switch_fs!(pub use {
  DirEntry,
  File,
  OpenOptions,
  ReadDir,
  canonicalize as canonicalize_raw,
  copy,
  create_dir,
  create_dir_all,
  hard_link,
  metadata,
  read,
  read_dir,
  read_link,
  read_to_string,
  remove_dir,
  remove_dir_all,
  remove_file,
  rename,
  set_permissions,
  symlink_metadata,
  write
});

/// Takes a path, returning the most compatible form of a path instead of UNC when on Windows.
#[inline]
pub fn simplified_path(path: &Path) -> &Path {
  #[cfg(windows)]
  let path = dunce::simplified(path);

  path
}

/// Behaves like [`canonicalize_raw`], but on Windows it outputs the
/// most compatible form of a path instead of UNC.
///
/// Uses [`dunce::simplified`] internally.
#[inline]
pub fn canonicalize<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
  let path = canonicalize_raw(path)?;

  #[cfg(windows)]
  let path = dunce::simplified(&path).to_owned();

  Ok(path)
}

//! Utility module re-exporting filesystem functions.
//!
//! If the `fs-err3` feature is enabled, all of these
//! items will point to functions from `fs-err` v3.
//!
//! Otherwise, if the `fs-err2` feature is enabled, all of these
//! items will point to functions from `fs-err` v2.
//!
//! Otherwise, all of these items will point to functions
//! from the standard library.
//!
//! Since the contents of this module may change depending on what features are enabled
//! for `singlefile` (which may be influenced by a dependency!) you should understand
//! the differences between `std::fs` and the different versions of `fs-err` and
//! consider whether importing from this module is appropriate.
//!
//! As such, the contents of this module are considered an implementation detail
//! and may have breaking changes between versions.

pub extern crate dunce;

use std::path::{Path, PathBuf};

cfg_if::cfg_if!{
  if #[cfg(feature = "fs-err3")] {
    pub extern crate fs_err3 as fs_err;
  } else if #[cfg(feature = "fs-err2")] {
    pub extern crate fs_err2 as fs_err;
  }
}

macro_rules! import_fs4 {
  ($vis:vis use { $($name:ident $(as $new_name:ident)?),* $(,)? }) => (
    cfg_if::cfg_if!{
      if #[cfg(feature = "fs-err3")] {
        #[doc(no_inline)]
        $vis use fs4::fs_err3::{$($name $(as $new_name)?),*};
      } else if #[cfg(feature = "fs-err2")] {
        #[doc(no_inline)]
        $vis use fs4::fs_err2::{$($name $(as $new_name)?),*};
      } else {
        #[doc(no_inline)]
        $vis use fs4::fs_std::{$($name $(as $new_name)?),*};
      }
    }
  );
}

macro_rules! import_fs {
  ($vis:vis use { $($name:ident $(as $new_name:ident)?),* $(,)? }) => (
    cfg_if::cfg_if!{
      if #[cfg(feature = "fs-err3")] {
        #[doc(no_inline)]
        $vis use fs_err3::{$($name $(as $new_name)?),*};
      } else if #[cfg(feature = "fs-err2")] {
        #[doc(no_inline)]
        $vis use fs_err2::{$($name $(as $new_name)?),*};
      } else {
        #[doc(no_inline)]
        $vis use std::fs::{$($name $(as $new_name)?),*};
      }
    }
  );
}

#[doc(no_inline)]
pub use fs4::{
  TryLockError,
  FsStats as Stats,
  statvfs as stats,
  allocation_granularity,
  available_space,
  free_space,
  total_space
};

import_fs4!(pub use {
  FileExt
});

#[doc(no_inline)]
pub use std::fs::{
  FileTimes,
  FileType,
  Metadata,
  Permissions
};

import_fs!(pub use {
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
pub fn canonicalize<P: AsRef<Path>>(path: P) -> std::io::Result<PathBuf> {
  let path = canonicalize_raw(path)?;

  #[cfg(windows)]
  let path = dunce::simplified(&path).to_owned();

  Ok(path)
}

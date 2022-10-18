//! Basic formats for treating files as plain bytes or UTF-8 text.

use super::FileFormat;

use std::hash::Hash;
use std::io::{self, Read, Write};



/// A [`FileFormat`] that treats files as plain bytes.
/// This file format is only usable with types like `Vec<u8>` or `Box<[u8]>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlainBytes;

impl<T> FileFormat<T> for PlainBytes where T: AsRef<[u8]>, Vec<u8>: Into<T> {
  type FormatError = io::Error;

  fn to_writer<W: Write>(&self, mut writer: W, value: &T) -> io::Result<()> {
    writer.write_all(value.as_ref()).map_err(From::from)
  }

  fn from_reader<R: Read>(&self, mut reader: R) -> io::Result<T> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf.into())
  }
}

/// A [`FileFormat`] that treats files as plain UTF-8 text.
/// This file format is only usable with types like `String` or `Box<str>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlainUtf8;

impl<T> FileFormat<T> for PlainUtf8 where T: AsRef<str>, String: Into<T> {
  type FormatError = io::Error;

  fn to_writer<W: Write>(&self, mut writer: W, value: &T) -> io::Result<()> {
    writer.write_all(value.as_ref().as_bytes()).map_err(From::from)
  }

  fn from_reader<R: Read>(&self, mut reader: R) -> io::Result<T> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    Ok(buf.into())
  }
}

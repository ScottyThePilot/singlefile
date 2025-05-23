//! Basic formats for treating files as plain bytes or UTF-8 text.

use super::{FileFormat, FileFormatUtf8};

use std::hash::Hash;
use std::io::{self, Read, Write};



/// A [`FileFormat`] that treats files as plain bytes.
/// This file format is only usable with types like `Vec<u8>` or `Box<[u8]>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlainBytes;

impl<T> FileFormat<T> for PlainBytes where T: AsRef<[u8]>, Vec<u8>: Into<T> {
  type FormatError = io::Error;

  #[inline]
  fn from_reader_buffered<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    self.from_reader(reader)
  }

  fn from_reader<R: Read>(&self, mut reader: R) -> io::Result<T> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf.into())
  }

  #[inline]
  fn to_writer_buffered<W: Write>(&self, writer: W, value: &T) -> io::Result<()> {
    self.to_writer(writer, value)
  }

  fn to_writer<W: Write>(&self, mut writer: W, value: &T) -> io::Result<()> {
    writer.write_all(value.as_ref())
  }

  fn to_buffer(&self, value: &T) -> Result<Vec<u8>, Self::FormatError> {
    Ok(value.as_ref().to_owned())
  }
}

/// A [`FileFormat`] that treats files as plain UTF-8 text.
/// This file format is only usable with types like `String` or `Box<str>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlainUtf8;

impl<T> FileFormat<T> for PlainUtf8 where T: AsRef<str>, String: Into<T> {
  type FormatError = io::Error;

  #[inline]
  fn from_reader_buffered<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    self.from_reader(reader)
  }

  fn from_reader<R: Read>(&self, mut reader: R) -> io::Result<T> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    Ok(buf.into())
  }

  #[inline]
  fn to_writer_buffered<W: Write>(&self, writer: W, value: &T) -> io::Result<()> {
    self.to_writer(writer, value)
  }

  fn to_writer<W: Write>(&self, mut writer: W, value: &T) -> io::Result<()> {
    writer.write_all(value.as_ref().as_bytes())
  }

  fn to_buffer(&self, value: &T) -> Result<Vec<u8>, Self::FormatError> {
    Ok(value.as_ref().to_owned().into_bytes())
  }
}

impl<T> FileFormatUtf8<T> for PlainUtf8 where T: AsRef<str>, String: Into<T> {
  fn from_string_buffer(&self, buf: &str) -> Result<T, Self::FormatError> {
    Ok(buf.to_owned().into())
  }

  fn to_string_buffer(&self, value: &T) -> Result<String, Self::FormatError> {
    Ok(value.as_ref().to_owned())
  }
}

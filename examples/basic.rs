#[macro_use]
extern crate serde;
extern crate singlefile;

use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use singlefile::{FileFormat, ContainerWritable};

use std::io::{Read, Write};

fn main() {
  // Create a new container, the data and data format must be specified somehow.
  let mut container = ContainerWritable::<Data, Json>::create_or_default("data.json", Json)
    .expect("failed to create data file container");
  println!("data: {:?}", container.get());

  // `Container` types implement `Deref` and `DerefMut`,
  // so you can operate on the contained value's methods and fields.
  // This only modifies the copy in memory, however.
  container.magic_number = 42;
  container.magic_string = ":^)".to_owned();
  container.is_magic = true;

  // You can write the in-memory copy to disk by calling `.commit()`.
  container.commit().expect("failed to commit data");
  println!("data: {:?}", container.get());

  // And you can replace the in-memory copy with the data on disk (or 'refresh' it) by calling `.refresh()`.
  container.refresh().expect("failed to refresh data");
  println!("data: {:?}", container.get());

  // You can explicitly close the contained file handle with `.close()`.
  // This is not strictly necessary, since dropping a container releases the file descriptor/handle automatically.
  container.close().expect("failed to close file container");
}

#[derive(Debug, Serialize, Deserialize)]
struct Data {
  magic_number: u32,
  magic_string: String,
  is_magic: bool
}

impl Default for Data {
  fn default() -> Self {
    Data {
      magic_number: 0,
      magic_string: "none".to_owned(),
      is_magic: false
    }
  }
}

#[derive(Debug)]
struct Json;

// You can implement a file format however you want,
// since `singlefile` is serialization-framework agnostic.
// This one uses `serde_json` and `serde` to write any value
// implementing `Serialize` and `Deserialize` to disk in JSON format.
impl<T> FileFormat<T> for Json
where T: Serialize + DeserializeOwned {
  type FormatError = serde_json::Error;

  fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
    serde_json::to_writer_pretty(writer, value).map_err(From::from)
  }

  fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    serde_json::from_reader(reader).map_err(From::from)
  }
}

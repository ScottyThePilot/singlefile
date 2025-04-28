#[macro_use]
extern crate serde;
extern crate singlefile;

use singlefile::container::ContainerWritable;

// You can implement a file format however you want,
// since `singlefile` is serialization-framework agnostic.
// This example uses the preset JSON format from singlefile-formats.
use singlefile_formats::data::json_serde::Json;

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

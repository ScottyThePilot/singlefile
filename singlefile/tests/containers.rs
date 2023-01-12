#[macro_use]
extern crate serde;
extern crate singlefile;

use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use singlefile::FileFormat;

use std::{fs, mem};
use std::io::{Read, Write};

#[test]
fn container_writable() {
  use singlefile::container::ContainerWritable;

  let temp_dir = tempfile::tempdir().unwrap();
  let path = temp_dir.path().join("data.json");

  let mut container = ContainerWritable::<Data, Json>::create_or_default(&path, Json)
    .expect("failed to create container for data.json");

  assert!(path.exists());
  assert_eq!(container.number, 0);

  container.number += 1;
  container.commit()
    .expect("failed to commit state to disk");

  assert_eq!(container.number, 1);

  mem::drop(container);

  fs::remove_file(path).unwrap();
  temp_dir.close().unwrap();
}

#[test]
#[cfg(feature = "shared")]
fn container_shared_writable() {
  use singlefile::container_shared::ContainerSharedWritable;

  use std::thread;
  use std::convert::Infallible;

  let temp_dir = tempfile::tempdir().unwrap();
  let path = temp_dir.path().join("data.json");

  let container = ContainerSharedWritable::<Data, Json>::create_or_default(&path, Json)
    .expect("failed to create container for data.json");

  let magic_number = container.operate(|data| data.number);
  assert_eq!(magic_number, 0);

  let container1 = container.clone();
  let t1 = thread::spawn(move || {
    container1.operate_mut_commit(|data| {
      data.number += 1;
      Ok::<(), Infallible>(())
    }).unwrap();
  });

  let container2 = container.clone();
  let t2 = thread::spawn(move || {
    container2.operate_mut_commit(|data| {
      data.number += 1;
      Ok::<(), Infallible>(())
    }).unwrap();
  });

  let container3 = container.clone();
  let t3 = thread::spawn(move || {
    container3.operate_mut_commit(|data| {
      data.number += 1;
      Ok::<(), Infallible>(())
    }).unwrap();
  });

  t1.join().unwrap();
  t2.join().unwrap();
  t3.join().unwrap();

  let magic_number = container.operate(|data| data.number);
  assert_eq!(magic_number, 3);

  mem::drop(container);

  fs::remove_file(path).unwrap();
  temp_dir.close().unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
struct Data {
  number: i32
}

impl Default for Data {
  fn default() -> Self {
    Data { number: 0 }
  }
}

#[derive(Debug)]
struct Json;

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

#[macro_use]
extern crate serde;
extern crate singlefile;

use singlefile_formats::data::json_serde::Json;

use std::{fs, mem};

#[test]
fn container() {
  use singlefile::container::StandardContainer;

  let temp_dir = tempfile::tempdir().unwrap();
  let path = temp_dir.path().join("data.json");

  let mut container = StandardContainer::<Data, Json>::create_or_default(&path, Json, Default::default())
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
fn container_shared() {
  use singlefile::container_shared::StandardContainerShared;

  use std::thread;
  use std::convert::Infallible;

  let temp_dir = tempfile::tempdir().unwrap();
  let path = temp_dir.path().join("data.json");

  let container = StandardContainerShared::<Data, Json>::create_or_default(&path, Json, Default::default())
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

#[tokio::test]
#[cfg(feature = "shared-async")]
async fn container_shared_async() {
  use singlefile::container_shared_async::StandardContainerSharedAsync;

  use std::convert::Infallible;

  let temp_dir = tempfile::tempdir().unwrap();
  let path = temp_dir.path().join("data.json");

  let container = StandardContainerSharedAsync::<Data, Json>::create_or_default(&path, Json, Default::default()).await
    .expect("failed to create container for data.json");

  let magic_number = container.operate(async |data| data.number).await;
  assert_eq!(magic_number, 0);

  let container1 = container.clone();
  let t1 = tokio::spawn(async move {
    container1.operate_mut_commit(async |data| {
      data.number += 1;
      Ok::<(), Infallible>(())
    }).await.unwrap();
  });

  let container2 = container.clone();
  let t2 = tokio::spawn(async move {
    container2.operate_mut_commit(async |data| {
      data.number += 1;
      Ok::<(), Infallible>(())
    }).await.unwrap();
  });

  let container3 = container.clone();
  let t3 = tokio::spawn(async move {
    container3.operate_mut_commit(async |data| {
      data.number += 1;
      Ok::<(), Infallible>(())
    }).await.unwrap();
  });

  t1.await.unwrap();
  t2.await.unwrap();
  t3.await.unwrap();

  let magic_number = container.operate(async |data| data.number).await;
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

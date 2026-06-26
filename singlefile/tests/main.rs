extern crate serde;
extern crate singlefile;
extern crate singlefile_formats;

use serde::{Serialize, Deserialize};
use singlefile::FileFormat;
use singlefile::container::Container;
use singlefile::container_shared::ContainerShared;
use singlefile::container_shared_async::ContainerSharedAsync;
use singlefile::manager::FileManager;
use singlefile::manager::atomic::AtomicFileSupport;

use std::{fs, io, mem};
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct Data {
  number: i32
}

impl Default for Data {
  fn default() -> Self {
    Data { number: 0 }
  }
}

macro_rules! test_group {
  ($name:ident, $format:expr, $extension:expr) => (
    mod $name {
      #[test]
      fn standard_container() {
        super::Tester::new($extension).test_container_standard($format);
      }

      #[test]
      fn standard_container_shared() {
        super::Tester::new($extension).test_container_shared_standard($format);
      }

      #[tokio::test]
      async fn standard_container_shared_async() {
        super::Tester::new($extension).test_container_shared_async_standard($format).await;
      }

      #[test]
      fn atomic_container() {
        super::Tester::new($extension).test_container_atomic($format);
      }

      #[test]
      fn atomic_container_shared() {
        super::Tester::new($extension).test_container_shared_atomic($format);
      }

      #[tokio::test]
      async fn atomic_container_shared_async() {
        super::Tester::new($extension).test_container_shared_async_atomic($format).await;
      }
    }
  );
}

test_group!(tests_cbor, singlefile_formats::data::cbor_serde::Cbor, "cbor");
test_group!(tests_json, singlefile_formats::data::json_serde::Json::<true>, "json");
test_group!(tests_toml, singlefile_formats::data::toml_serde::Toml::<true>, "toml");
test_group!(tests_ron, singlefile_formats::data::ron_serde::Ron::default(), "ron");
test_group!(tests_ron_pretty, singlefile_formats::data::ron_serde::RonPretty::default(), "ron");

#[derive(Debug, Clone)]
struct Tester {
  tempdir: Arc<tempfile::TempDir>,
  extension: &'static str
}

impl AtomicFileSupport for Tester {
  fn pick_temporary_file_location(&mut self, _: &Path) -> io::Result<PathBuf> {
    Ok(self.data_path_temp())
  }
}

impl Tester {
  fn new(extension: &'static str) -> Self {
    Tester {
      tempdir: Arc::new(tempfile::tempdir().unwrap()),
      extension
    }
  }

  fn data_path(&self) -> PathBuf {
    self.tempdir.path().join(format!("data.{}", self.extension))
  }

  fn data_path_temp(&self) -> PathBuf {
    self.tempdir.path().join(format!("data.temp.{}", self.extension))
  }

  fn test_container_standard<F>(&self, format: F)
  where F: FileFormat<Data>, F::FormatError: Send + Sync + 'static {
    use singlefile::manager::standard::*;
    self.test_container::<StandardManager<F>, F>(format, StandardManagerOptions::default());
  }

  fn test_container_shared_standard<F>(&self, format: F)
  where F: FileFormat<Data> + Send + Sync + 'static, F::FormatError: Send + Sync + 'static {
    use singlefile::manager::standard::*;
    self.test_container_shared::<StandardManager<F>, F>(format, StandardManagerOptions::default());
  }

  async fn test_container_shared_async_standard<F>(&self, format: F)
  where F: FileFormat<Data> + Send + Sync + 'static, F::FormatError: Send + Sync + 'static {
    use singlefile::manager::standard::*;
    self.test_container_shared_async::<StandardManager<F>, F>(format, StandardManagerOptions::default()).await;
  }

  fn test_container_atomic<F>(&self, format: F)
  where F: FileFormat<Data>, F::FormatError: Send + Sync + 'static {
    use singlefile::manager::atomic::*;
    self.test_container::<AtomicManager<F, Self>, F>(format, AtomicManagerOptions::new(self.clone()));
  }

  fn test_container_shared_atomic<F>(&self, format: F)
  where F: FileFormat<Data> + Send + Sync + 'static, F::FormatError: Send + Sync + 'static {
    use singlefile::manager::atomic::*;
    self.test_container_shared::<AtomicManager<F, Self>, F>(format, AtomicManagerOptions::new(self.clone()));
  }

  async fn test_container_shared_async_atomic<F>(&self, format: F)
  where F: FileFormat<Data> + Send + Sync + 'static, F::FormatError: Send + Sync + 'static {
    use singlefile::manager::atomic::*;
    self.test_container_shared_async::<AtomicManager<F, Self>, F>(format, AtomicManagerOptions::new(self.clone())).await;
  }

  fn test_container<M, F>(&self, format: F, options: M::Options)
  where
    M: FileManager<Data, Format = F>,
    M::Error: Send + Sync + 'static,
    F: FileFormat<Data>
  {
    let path = self.data_path();

    let mut container = Container::<Data, M>::create_or_default(&path, format, options)
      .expect("failed to create container");

    assert!(path.exists());

    assert_eq!(container.number, 0);

    container.refresh()
      .expect("failed to refresh container state");

    assert_eq!(container.number, 0);

    container.number += 1;
    container.commit()
      .expect("failed to commit state to disk");

    assert_eq!(container.number, 1);

    container.refresh()
      .expect("failed to refresh container state");

    assert_eq!(container.number, 1);

    mem::drop(container);

    fs::remove_file(path).unwrap();
  }

  fn test_container_shared<M, F>(&self, format: F, options: M::Options)
  where
    M: FileManager<Data, Format = F> + Send + Sync + 'static,
    M::Error: Send + Sync + 'static,
    F: FileFormat<Data> + Send + Sync + 'static
  {
    let path = self.data_path();

    let container = ContainerShared::<Data, M>::create_or_default(&path, format, options)
      .expect("failed to create container");

        assert!(path.exists());

    let magic_number = container.operate(|data| data.number);
    assert_eq!(magic_number, 0);

    container.refresh()
      .expect("failed to refresh container state");

    let magic_number = container.operate(|data| data.number);
    assert_eq!(magic_number, 0);

    let container1 = container.clone();
    let t1 = std::thread::spawn(move || {
      container1.operate_mut_commit(|data| {
        data.number += 1;
        Ok::<(), Infallible>(())
      }).unwrap();
    });

    let container2 = container.clone();
    let t2 = std::thread::spawn(move || {
      container2.operate_mut_commit(|data| {
        data.number += 1;
        Ok::<(), Infallible>(())
      }).unwrap();
    });

    let container3 = container.clone();
    let t3 = std::thread::spawn(move || {
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

    container.refresh()
      .expect("failed to refresh container state");

    let magic_number = container.operate(|data| data.number);
    assert_eq!(magic_number, 3);

    mem::drop(container);

    fs::remove_file(path).unwrap();
  }

  async fn test_container_shared_async<M, F>(&self, format: F, options: M::Options)
  where
    M: FileManager<Data, Format = F> + Send + Sync + 'static,
    M::Options: Send, M::Error: Send + Sync + 'static,
    F: FileFormat<Data> + Send + Sync + 'static,
    F::FormatError: Send
  {
    let path = self.data_path();

    let container = ContainerSharedAsync::<Data, M>::create_or_default(&path, format, options).await
      .expect("failed to create container");

    assert!(path.exists());

    let magic_number = container.operate(async |data| data.number).await;
    assert_eq!(magic_number, 0);

    container.refresh().await
      .expect("failed to refresh container state");

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

    container.refresh().await
      .expect("failed to refresh container state");

    let magic_number = container.operate(async |data| data.number).await;
    assert_eq!(magic_number, 3);

    mem::drop(container);

    fs::remove_file(path).unwrap();
  }
}

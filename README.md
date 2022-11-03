# SingleFile
This library is designed to be a dead-simple way of accessing and manipulating files,
treating those files as if they represent some Rust value.

## Usage
`singlefile` provides a generic `Container` type, along with type alias variants for different use cases.
`Container` is named so to indicate that it contains and manages a file and a value.

```rust
// A readable, writable container
use singlefile::container::ContainerWritable;

#[derive(Serialize, Deserialize, Default)]
struct MyData {
  magic_number: i32
}

// Attempts to open 'my_data.json', creating it from default if it does not exist,
// expecting data that the `Json` format can decode into `MyData`.
let mut my_container = ContainerWritable::<MyData, Json>::create_or_default("my_data.json", Json)?;
// For regular `Container`s, `Deref` and `DerefMut` can be used to access the contained type
println!("magic_number: {}", my_container.magic_number); // 0 (as long as the file didn't exist before)
my_container.magic_number += 1;
// Write the new state of `MyData` to disk
my_container.commit()?;
```

We'd then expect the resulting `my_data.json` to look like:

```json
{
  "magic_number": 1
}
```

## Shared and async containers
`singlefile` also provides a `ContainerShared` type that can be used from multiple threads, as well as
a `ContainerAsync` that can be used from multiple threads and spawns its operations asynchronously.
Currently, `ContainerAsync` can only be guaranteed to work alongside Tokio.

The shared container types can be enabled with the `shared` cargo feature.
The async container types can be enabled with the `shared-async` cargo feature.

```rust
// A readable, writable container with multiple-ownership
use singlefile::container_shared::ContainerSharedWritable;

// `ContainerShared` types may be cloned cheaply, they behave like `Arc`s
let my_container = ContainerSharedWritable::<MyData, Json>::create_or_default("my_data.json", Json)?;

// Get access to the contained `MyData`, increment it, and commit changes to disk
std::thread::spawn(move || {
  my_container.operate_mut_commit(|my_data| {
    my_data.magic_number += 1;
    Ok::<(), Infallible>(())
  });
});
```

## File formats
`singlefile` is serialization framework-agnostic, so you will need a `FileFormat` adapter
before you are able to read and write a given file format to disk.

Here is how you'd write a `Json` adapter for the above examples, using `serde`.

```rust
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use singlefile::FileFormat;
use std::io::{Read, Write};

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
```

# SingleFile

This library is designed to be a dead-simple way of reading and writing your rust values to and from disk.
It greatly reduces the boilerplate necessary for using simple config files or basic JSON datastores.

See the docs for each crate for more info:

Crate name           | Crates.io page                         | Docs.rs page
---------------------|----------------------------------------|-------------------------------------------
`singlefile`         | [![Crate][crates-img-1]][crates-url-1] | [![Documentation][docs-img-1]][docs-url-1]
`singlefile-formats` | [![Crate][crates-img-2]][crates-url-2] | [![Documentation][docs-img-2]][docs-url-2]

## Example

```rust
// A JSON file format, utilizing serde
use singlefile_formats::json_serde::Json;
// A readable, writable container
use singlefile::container::ContainerWritable;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
struct MyData {
  magic_number: i32
}

// Attempts to open 'my_data.json', creating it from default if it does not exist,
// expecting data that the `Json` format can decode into `MyData`
let mut my_container = ContainerWritable::<MyData, Json>::create_or_default("my_data.json", Json)?;
// For regular `Container`s, `Deref` and `DerefMut` can be used to access the contained type
println!("magic_number: {}", my_container.magic_number); // 0 (as long as the file didn't exist before)
my_container.magic_number += 1;
// Write the new state of `MyData` to disk
my_container.commit()?;
```

[crates-img-1]: https://img.shields.io/crates/v/singlefile.svg
[crates-img-2]: https://img.shields.io/crates/v/singlefile-formats.svg
[crates-url-1]: https://crates.io/crates/singlefile
[crates-url-2]: https://crates.io/crates/singlefile-formats

[docs-img-1]: https://docs.rs/singlefile/badge.svg
[docs-img-2]: https://docs.rs/singlefile-formats/badge.svg
[docs-url-1]: https://docs.rs/singlefile
[docs-url-2]: https://docs.rs/singlefile-formats

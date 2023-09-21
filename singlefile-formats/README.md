# SingleFile Formats
[![Crate](https://img.shields.io/crates/v/singlefile-formats.svg)](https://crates.io/crates/singlefile-formats)
[![Documentation](https://docs.rs/singlefile-formats/badge.svg)](https://docs.rs/singlefile-formats)

This library provides a number of default `FileFormat` implementations
for use within [`singlefile`](https://crates.io/crates/singlefile).

# Features
By default, no features are enabled.

- `cbor-serde`: Enables the `Cbor` file format for use with `serde` types.
- `json-serde`: Enables the `Json` file format for use with `serde` types.
- `toml-serde`: Enables the `Toml` file format for use with `serde` types.
- `bzip`: Enables the `BZip2` compression format.
- `flate`: Enables the `Deflate`, `Gz`, and `ZLib` compression formats.
- `xz`: Enables the `Xz` compression format.

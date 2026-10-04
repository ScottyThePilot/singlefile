# SingleFile Formats
[![Crate](https://img.shields.io/crates/v/singlefile-formats.svg)](https://crates.io/crates/singlefile-formats)
[![Documentation](https://docs.rs/singlefile-formats/badge.svg)](https://docs.rs/singlefile-formats)

This library provides a number of default `FileFormat` implementations
for use within [`singlefile`](https://crates.io/crates/singlefile).

## Features
By default, no features are enabled.

- `bincode`: Enables the `Bincode` file format.
- `bincode-serde`: Enables the `BincodeSerde` file format for use with `serde` types.
- `cbor-serde`: Enables the `Cbor` file format for use with `serde` types.
- `json-serde`: Enables the `Json` file format for use with `serde` types.
- `ron-serde`: Enables the `Ron` file format for use with `serde` types.
- `toml-serde`: Enables the `Toml` file format for use with `serde` types.
- `bzip`: Enables the `BZip2` compression format. See `CompressionFormat` for more info.
- `bzip-rust`: Enables the `libbz2-rs-sys` feature for `bzip2`.
- `flate`: Enables the `Deflate`, `Gz`, and `ZLib` compression formats. See `CompressionFormat` for more info.
- `xz`: Enables the `Xz` compression format. See `CompressionFormat` for more info.

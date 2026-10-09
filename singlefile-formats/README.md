# SingleFile Formats
[![Crate](https://img.shields.io/crates/v/singlefile-formats.svg)](https://crates.io/crates/singlefile-formats)
[![Documentation](https://docs.rs/singlefile-formats/badge.svg)](https://docs.rs/singlefile-formats)

This library provides a number of default `FileFormat` implementations
for use within [`singlefile`](https://crates.io/crates/singlefile).

## Features
By default, only the `compression` feature is enabled.

- `compression`: Enables the `compression` module. Enabled by default.
- `utils-serde`: Enables the `utils_serde` module.
- `base64`: Enables the `Base64` file format.
- `bincode1`: Enables the `Bincode` file format.
- `bincode2`: Enables the `Bincode` file format.
- `bincode2-serde`: Enables the `BincodeSerde` file format for use with `serde` types.
- `bincode-reloaded`: Enables the `Bincode` file format.
- `bincode-reloaded-serde`: Enables the `BincodeSerde` file format for use with `serde` types.
- `cbor-serde`: Enables the `Cbor` file format for use with `serde` types.
- `json-serde`: Enables the `Json` file format for use with `serde` types.
- `ron-serde`: Enables the `Ron` file format for use with `serde` types.
- `toml-serde`: Enables the `Toml` file format for use with `serde` types.
- `bzip`: Enables the `BZip2` compression format. See `CompressionFormat` for more info.
- `flate`: Enables the `Deflate`, `Gz`, and `ZLib` compression formats. See `CompressionFormat` for more info.
- `xz`: Enables the `Xz` compression format. See `CompressionFormat` for more info.

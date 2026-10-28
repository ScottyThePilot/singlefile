//! Provides a number of data formats as [`FileFormat`][singlefile::FileFormat]s.
//!
//! For each format, the original crate is available as an alias, `pub extern crate ... as original`.

pub mod base64;
pub mod bincode1;
pub mod bincode2;
pub mod bincode_reloaded;
pub mod borsh;
pub mod cbor_serde;
pub mod json_serde;
pub mod ron_serde;
pub mod toml_serde;

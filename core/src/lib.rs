#![feature(arbitrary_self_types, decl_macro, proc_macro_hygiene)]
//! # Rocket-Config - Core API Documentation
//!
//! Hello, and welcome to the core Rocket-Config API documentation!
//!
//! Rocket-Config is a Rust library providing a plugin for [Rocket] loading and
//! managing configuration files for [Rocket].
//!
//! It allows two configuration file formats: [YAML] and [JSON].
//! Deserialization is done using [serde] and specialized packages [serde_json]
//! and [serde_yaml].
//!
//! # Libraries
//!
//! Rocket-Config's functionality is split into two crates:
//!
//!   1. Core - This core library. Needed by every Rocket application using
//! rocket-config.
//!   2. [Codegen] - Provides useful code generation functionality for many
//! Rocket applications. Completely optional.
//!
//! ## Usage
//!
//! First, depend on `rocket-config` in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rocket-config = "0.0.1"
//! ```
//!
//! Then, add the following to the top of your `main.rs` file:
//!
//! ```rust
//! #![feature(proc_macro_hygiene, decl_macro)]
//!
//! #[macro_use] extern crate rocket_config;
//!
//! // ...
//! ```
//!
//! [Codegen]: ../rocket_config_codegen/index.html
//! [JSON]: http://json.org
//! [Rocket]: https://rocket.rs/
//! [serde]: https://serde.rs/
//! [serde_json]: https://docs.serde.rs/serde_json/
//! [serde_yaml]: https://docs.serde.rs/serde_yaml/
//! [YAML]: http://yaml.org

#![warn(rust_2018_idioms)]

#[allow(unused_imports)] #[macro_use] extern crate rocket_config_codegen;
#[doc(hidden)] pub use rocket_config_codegen::*;

#[macro_use] extern crate lazy_static;
#[cfg(test)] #[macro_use] extern crate serde_json;
#[cfg(test)] extern crate tempfile;

mod configuration;
mod constants;
pub mod error;
mod factory;
mod result;
mod value;

pub use configuration::Configuration;
pub use factory::Factory;
pub use result::Result;
pub use value::*;
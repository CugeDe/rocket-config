#![feature(proc_macro_diagnostic)]
#![recursion_limit="256"]

//! # Rocket-Config - Code Generation
//!
//! This crate implements the code generation portions of Rocket-config.
//! This includes procedural macros.
//!
//! ## Procedural Macros
//!
//! This crate implements the following procedural macros:
//!
//! * **configuration**
//!
//! The syntax for the `configuration` macro is:
//!
//! <pre>
//! macro := configuration!(CONFIGURATION_FILE_STEM)
//! </pre>
//!
//! ## Usage
//!
//! You **_should not_** directly depend on this library. To use the macros,
//! it suffices to depend on `rocket-config` in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rocket-config = "0.0.1"
//! ```
//!
//! And to import the macros via `#[macro_use]` in the crate root:
//!
//! ```rust
//! #![feature(proc_macro_hygiene, decl_macro)]
//!
//! #[macro_use] extern crate rocket_config;
//!
//! configuration!("test");
//!
//! // ...
//! ```
//!
//! Or, alternatively, selectively import from the top-level scope:
//!
//! ```rust
//! #![feature(proc_macro_hygiene, decl_macro)]
//!
//! extern crate rocket_config;
//!
//! use rocket_config::configuration;
//!
//! configuration!("test");
//!
//! // ...
//! ```

#![warn(rust_2018_idioms)]

#[macro_use] extern crate quote;
extern crate proc_macro;

mod configuration;

#[allow(unused_imports)]
use proc_macro::TokenStream;

/// The procedural macro for the `configuration` function-like macro.
#[proc_macro]
pub fn configuration(input: TokenStream) -> TokenStream {
    configuration::configuration_function(input)
}

//! # jason-RS
//!
//! [![Crates.io](https://img.shields.io/crates/v/jason-rs)](https://crates.io/crates/jason-rs)
//! [![Docs](https://docs.rs/jason-rs/badge.svg)](https://docs.rs/jason-rs)
//! [![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
//!
//! **jason** is a lightweight JSON templating tool that transforms reusable `.jason` template files into standard JSON. Build modular and composable JSON structures with parameter support and file inclusion.
//!
//! ## Features
//!
//! - Template Parameters — Define reusable templates with named parameters
//! - File Inclusion — Compose JSON from multiple `.jason` files
//! - 1:1 Conversion — Clean mapping from `.jason` syntax to standard JSON
//! - Library-First — Designed for seamless integration into Rust projects
//!
//! ## Usage
//!
//! Add `jason-rs` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! jason-rs = "0.1"
//! ```
//!
//! Parse a Jason file to JSON:
//!
//! ```rust
//! use jason_rs::jason_to_json;
//!
//! fn main() {
//!     let json = jason_to_json("./Page.jason").unwrap();
//!     println!("{}", json);
//! }
//! ```
//!
//! ## Syntax Overview
//!
//! | Syntax | Description |
//! |--------|-------------|
//! | `(param1, param2)` | Define template parameters |
//! | `#param` | Reference a parameter value |
//! | `<./File.jason>` | Include another Jason file |
//! | `<./File.jason \| arg1, arg2>` | Include with arguments |
//!
//! Parses a `.jason` file at the given path and returns the resulting JSON as a `String`.

/// The `jason` module contains all public functions for converting `.jason` files
/// into JSON, YAML, or TOML.

pub mod jason;
mod lexer; 
mod token;
mod parser;
mod context;
mod json_compiler;
mod template;
mod astnode;
pub use jason::*;


#[cfg(test)]
mod tests {
    // Import the outer module’s items
    use super::*;

    #[test]
    fn test_add_positive() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-2, -3), -5);
    }

    #[test]
    #[should_panic]
    fn test_panic_example() {
        panic!("This should panic");
    }
}










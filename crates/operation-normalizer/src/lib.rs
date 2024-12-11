//! # Grafbase Operation Normalizer
//!
//! A library to modify a incoming query so we can match similar queries to be the
//! same, even if they have differences in their string representation. This is achieved
//! by doing the following steps:
//!
//! - Removal of all hard-coded arguments for the following types:
//!   - String (replace with "")
//!   - Float (replace with 0.0)
//!   - Int (replace with 0)
//!   - List (replace with [])
//!   - Object (replace with {})
//! - Leave parameters, enums and booleans as-is
//! - Remove all fragments not used in the query
//! - Remove all comments
//! - Reorder fields, arguments, selections in alphabetic order
//! - Parse and render, removing extra whitespace and other stylistic things

#![deny(missing_docs)]

mod normalize;
mod sanitize;

#[cfg(test)]
mod tests;

pub use normalize::normalize;
pub use sanitize::sanitize;

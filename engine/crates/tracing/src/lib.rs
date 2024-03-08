#![forbid(missing_docs)]

//! Grafbase [tracing](https://docs.rs/tracing/latest/tracing/) integration

/// Tracing configuration properties
pub mod config;
/// Potential errors from this crate
pub mod error;
/// Otel integration
pub mod otel;
/// Spans that are represented using types
pub mod span;
/// [Tower](https://docs.rs/tower/latest/tower/) integration
#[cfg(feature = "tower")]
pub mod tower;

#![allow(unused_crate_dependencies)]
//! Grafbase [tracing](https://docs.rs/tracing/latest/tracing/) integration

pub use gateway_config::telemetry as config;
/// Potential errors from this crate
pub mod error;
pub mod gql_response_status;
pub mod grafbase_client;
pub mod http;
pub mod metrics;
/// Otel integration
pub mod otel;
/// Spans that are represented using types
pub mod span;
/// [Tower](https://docs.rs/tower/latest/tower/) integration
#[cfg(feature = "tower")]
pub mod tower;

pub(crate) const SCOPE: &str = "grafbase";
pub(crate) const SCOPE_VERSION: &str = "1.0";

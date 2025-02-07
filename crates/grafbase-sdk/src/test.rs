//! Test utilities for running and interacting with a GraphQL gateway.
//!
//! This module provides functionality for:
//! - Configuring and starting a gateway instance
//! - Executing GraphQL queries against the gateway
//! - Building and loading extensions

mod config;
mod runner;

pub use config::{TestConfig, TestConfigBuilder};
pub use grafbase_sdk_mock::{DynamicSchema, DynamicSubgraph};
pub use runner::{QueryBuilder, TestRunner};

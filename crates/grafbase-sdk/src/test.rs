//! Test utilities for running and interacting with a GraphQL gateway.
//!
//! This module provides functionality for:
//! - Configuring and starting a gateway instance
//! - Executing GraphQL queries against the gateway
//! - Building and loading extensions

mod config;
mod gateway;
mod request;

pub use config::LogLevel;
pub use gateway::{TestGateway, TestGatewayBuilder};
pub use grafbase_sdk_mock::{GraphqlSubgraph, GraphqlSubgraphBuilder, VirtualSubgraph};
pub use request::{GraphqlCollectedStreamingResponse, GraphqlRequest, GraphqlResponse, GraphqlStreamingResponse};

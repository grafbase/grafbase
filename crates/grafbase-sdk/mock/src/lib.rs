//! Provides a dynamic GraphQL schema and subgraph implementation that can be built and executed at runtime.
//!
//! This crate allows creating GraphQL schemas dynamically from SDL (Schema Definition Language) strings
//! and executing queries against them. It also provides functionality for running mock GraphQL servers
//! using these dynamic schemas.

#![deny(missing_docs)]

mod builder;
mod entity_resolver;
mod resolver;
mod server;

use std::sync::Arc;

pub use builder::DynamicSchemaBuilder;
pub use server::MockGraphQlServer;

/// A dynamic GraphQL schema that can be built and executed at runtime.
#[derive(Debug)]
pub struct DynamicSchema {
    schema: async_graphql::dynamic::Schema,
    sdl: String,
}

impl DynamicSchema {
    /// Creates a builder for constructing a new dynamic subgraph schema from SDL.
    ///
    /// # Arguments
    ///
    /// * `sdl` - GraphQL schema definition language string to build from
    pub fn builder(sdl: impl AsRef<str>) -> DynamicSchemaBuilder {
        DynamicSchemaBuilder::new(sdl.as_ref())
    }

    /// Executes a GraphQL request against this schema.
    pub async fn execute(&self, request: async_graphql::Request) -> async_graphql::Response {
        self.schema.execute(request).await
    }

    /// Returns the SDL (Schema Definition Language) string for this schema.
    pub fn sdl(&self) -> &str {
        &self.sdl
    }
}

/// A dynamic subgraph implementation that can be started as a mock GraphQL server.
#[derive(Debug)]
pub struct DynamicSubgraph {
    schema: DynamicSchema,
    name: String,
}

impl DynamicSubgraph {
    /// Starts this subgraph as a mock GraphQL server.
    ///
    /// Returns a handle to the running server that can be used to stop it.
    pub async fn start(self) -> MockGraphQlServer {
        MockGraphQlServer::new(self.name, Arc::new(self.schema)).await
    }
}

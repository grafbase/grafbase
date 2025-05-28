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

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub use builder::DynamicSchemaBuilder;
pub use server::MockGraphQlServer;

/// A dynamic GraphQL schema that can be built and executed at runtime.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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

/// A subgraph that only contains extension definitions. We do not spawn a GraphQL server for this subgraph.
#[derive(Debug, Clone)]
pub struct ExtensionOnlySubgraph {
    schema: DynamicSchema,
    name: String,
    extension_path: PathBuf,
}

impl ExtensionOnlySubgraph {
    /// Returns the SDL (Schema Definition Language) string for this schema
    pub fn sdl(&self) -> &str {
        self.schema.sdl()
    }

    /// Returns the name of this subgraph
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the path to the extension definitions for this subgraph
    pub fn extension_path(&self) -> &Path {
        &self.extension_path
    }
}

/// A mock subgraph that can either be a full dynamic GraphQL service or just extension definitions.
#[derive(Debug, Clone)]
pub enum MockSubgraph {
    /// A full dynamic subgraph that can be started as a GraphQL server
    Dynamic(DynamicSubgraph),
    /// A subgraph that only contains extension definitions and is not started as a server
    ExtensionOnly(ExtensionOnlySubgraph),
}

impl From<DynamicSubgraph> for MockSubgraph {
    fn from(subgraph: DynamicSubgraph) -> Self {
        MockSubgraph::Dynamic(subgraph)
    }
}

impl From<ExtensionOnlySubgraph> for MockSubgraph {
    fn from(subgraph: ExtensionOnlySubgraph) -> Self {
        MockSubgraph::ExtensionOnly(subgraph)
    }
}

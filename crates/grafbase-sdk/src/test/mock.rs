mod builder;
mod entity_resolver;
mod resolver;
mod server;

use std::{net::SocketAddr, sync::Arc};

pub use builder::DynamicSchemaBuilder;
pub use server::MockGraphQlServer;
use url::Url;

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

    async fn execute(&self, request: async_graphql::Request) -> async_graphql::Response {
        self.schema.execute(request).await
    }

    fn sdl(&self) -> &str {
        &self.sdl
    }
}

/// A dynamic subgraph implementation that can be started as a mock GraphQL server.
#[derive(Debug)]
pub struct DynamicSubgraph {
    schema: DynamicSchema,
    name: String,
    listen_address: SocketAddr,
}

impl DynamicSubgraph {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn start(self) -> MockGraphQlServer {
        MockGraphQlServer::new(Arc::new(self.schema), self.listen_address)
    }

    pub(crate) fn sdl(&self) -> &str {
        self.schema.sdl()
    }

    pub(crate) fn url(&self) -> Url {
        format!("http://{}", self.listen_address).parse().unwrap()
    }
}

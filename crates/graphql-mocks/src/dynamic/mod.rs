mod builder;
mod entity_resolvers;
mod resolvers;

use std::sync::Arc;

pub use self::{
    builder::DynamicSchemaBuilder,
    entity_resolvers::{EntityResolver, EntityResolverContext},
    resolvers::Resolver,
};

pub use async_graphql::{dynamic::ResolverContext, ServerError};

use crate::MockGraphQlServer;

pub struct DynamicSubgraph {
    schema: DynamicSchema,
    name: String,
}

impl super::Subgraph for DynamicSubgraph {
    fn name(&self) -> String {
        self.name.clone()
    }

    async fn start(self) -> crate::MockGraphQlServer {
        MockGraphQlServer::new(self.schema).await
    }
}

/// async-graphql powered dynamic schemas for tests.
///
/// Occasionally its just easier to write SDL & resolvers, this lets you do that.
pub struct DynamicSchema {
    schema: async_graphql::dynamic::Schema,
    sdl: String,
}

impl DynamicSchema {
    pub fn builder(sdl: impl AsRef<str>) -> DynamicSchemaBuilder {
        DynamicSchemaBuilder::new(sdl.as_ref())
    }
}

#[async_trait::async_trait]
impl super::Schema for DynamicSchema {
    async fn execute(
        &self,
        _headers: Vec<(String, String)>,
        request: async_graphql::Request,
    ) -> async_graphql::Response {
        self.schema.execute(request).await
    }

    fn execute_stream(
        &self,
        request: async_graphql::Request,
        session_data: Option<Arc<async_graphql::Data>>,
    ) -> futures::stream::BoxStream<'static, async_graphql::Response> {
        if let Some(session_data) = session_data {
            Box::pin(self.schema.execute_stream_with_session_data(request, session_data))
        } else {
            Box::pin(self.schema.execute_stream(request))
        }
    }

    fn sdl(&self) -> String {
        self.sdl.clone()
    }
}

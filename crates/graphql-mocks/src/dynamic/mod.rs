mod builder;
mod resolvers;

pub use builder::DynamicSchemaBuilder;

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
    ) -> futures::stream::BoxStream<'static, async_graphql::Response> {
        Box::pin(self.schema.execute_stream(request))
    }

    fn sdl(&self) -> String {
        self.sdl.clone()
    }
}

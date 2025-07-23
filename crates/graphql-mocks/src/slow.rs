use std::sync::Arc;

use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};

use crate::MockGraphQlServer;

pub struct SlowSchema {
    schema: Schema<Query, EmptyMutation, EmptySubscription>,
}

impl crate::Subgraph for SlowSchema {
    fn name(&self) -> String {
        "slow".to_string()
    }

    async fn start(self) -> MockGraphQlServer {
        MockGraphQlServer::new(self).await
    }
}

impl Default for SlowSchema {
    fn default() -> Self {
        Self {
            schema: Schema::build(Query, EmptyMutation, EmptySubscription)
                .enable_federation()
                .finish(),
        }
    }
}

struct Query;

#[Object]
impl Query {
    async fn delay(&self, ms: u32) -> u32 {
        tokio::time::sleep(tokio::time::Duration::from_millis(ms.into())).await;
        ms
    }

    async fn nullable_delay(&self, ms: u32) -> Option<u32> {
        tokio::time::sleep(tokio::time::Duration::from_millis(ms.into())).await;
        Some(ms)
    }
}

#[async_trait::async_trait]
impl crate::Schema for SlowSchema {
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
        async_graphql::Executor::execute_stream(&self.schema, request, session_data)
    }

    fn sdl(&self) -> String {
        self.schema
            .sdl_with_options(async_graphql::SDLExportOptions::new().federation())
    }
}

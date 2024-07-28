use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};

use crate::MockGraphQlServer;

#[derive(Default)]
pub struct SlowSchema;

impl crate::Subgraph for SlowSchema {
    fn name(&self) -> String {
        "slow".to_string()
    }

    async fn start(self) -> MockGraphQlServer {
        MockGraphQlServer::new(self).await
    }
}

impl SlowSchema {
    fn schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
        Schema::build(Query, EmptyMutation, EmptySubscription)
            .enable_federation()
            .finish()
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
        Self::schema().execute(request).await
    }

    fn execute_stream(
        &self,
        request: async_graphql::Request,
    ) -> futures::stream::BoxStream<'static, async_graphql::Response> {
        Box::pin(Self::schema().execute_stream(request))
    }

    fn sdl(&self) -> String {
        Self::schema().sdl_with_options(async_graphql::SDLExportOptions::new().federation())
    }
}

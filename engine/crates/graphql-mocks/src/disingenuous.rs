use async_graphql::{EmptyMutation, EmptySubscription, Json, Object};

pub struct DisingenuousSchema {
    sdl: String,
}

impl DisingenuousSchema {
    pub fn with_sdl(sdl: impl AsRef<str>) -> Self {
        Self {
            sdl: sdl.as_ref().to_string(),
        }
    }
}

#[async_trait::async_trait]
impl super::Schema for DisingenuousSchema {
    async fn execute(
        &self,
        _headers: Vec<(String, String)>,
        request: async_graphql::Request,
    ) -> async_graphql::Response {
        async_graphql::Schema::build(Query, EmptyMutation, EmptySubscription)
            .finish()
            .execute(request)
            .await
    }

    fn execute_stream(
        &self,
        request: async_graphql::Request,
    ) -> futures::stream::BoxStream<'static, async_graphql::Response> {
        Box::pin(
            async_graphql::Schema::build(Query, EmptyMutation, EmptySubscription)
                .finish()
                .execute_stream(request),
        )
    }

    fn sdl(&self) -> String {
        self.sdl.clone()
    }
}

pub struct Query;

#[Object]
impl Query {
    async fn echo(&self, input: Json<serde_json::Value>) -> Json<serde_json::Value> {
        input
    }
}

use std::sync::Arc;

use async_graphql::{
    EmptyMutation, EmptySubscription, Enum, ID, InputObject, Json, MaybeUndefined, Object, SimpleObject,
};
use crossbeam_queue::SegQueue;
use http::{HeaderMap, HeaderName, HeaderValue};

/// A schema that just echoes stuff back at you.
///
/// Useful for testing inputs & outputs
pub struct EchoSchema {
    schema: async_graphql::Schema<Query, EmptyMutation, EmptySubscription>,
}

impl crate::Subgraph for EchoSchema {
    fn name(&self) -> String {
        "echo".to_string()
    }
    async fn start(self) -> crate::MockGraphQlServer {
        crate::MockGraphQlServer::new(self).await
    }
}

impl Default for EchoSchema {
    fn default() -> Self {
        let schema = async_graphql::Schema::build(
            Query {
                headers: Default::default(),
                response_headers: Default::default(),
            },
            EmptyMutation,
            EmptySubscription,
        )
        .finish();
        Self { schema }
    }
}

#[async_trait::async_trait]
impl super::Schema for EchoSchema {
    async fn execute(
        &self,
        headers: Vec<(String, String)>,
        request: async_graphql::Request,
    ) -> async_graphql::Response {
        let response_headers = Arc::new(SegQueue::new());

        let response = async_graphql::Schema::build(
            Query {
                headers,
                response_headers: response_headers.clone(),
            },
            EmptyMutation,
            EmptySubscription,
        )
        .finish()
        .execute(request)
        .await;

        let mut headers = HeaderMap::new();

        while let Some((key, value)) = response_headers.pop() {
            headers.insert(
                HeaderName::from_bytes(key.as_bytes()).unwrap(),
                HeaderValue::from_bytes(value.as_bytes()).unwrap(),
            );
        }

        response.http_headers(headers)
    }

    fn execute_stream(
        &self,
        request: async_graphql::Request,
        session_data: Option<Arc<async_graphql::Data>>,
    ) -> futures::stream::BoxStream<'static, async_graphql::Response> {
        async_graphql::Executor::execute_stream(&self.schema, request, session_data)
    }

    fn sdl(&self) -> String {
        self.schema.sdl_with_options(async_graphql::SDLExportOptions::new())
    }
}

pub struct Query {
    headers: Vec<(String, String)>,
    response_headers: Arc<SegQueue<(String, String)>>,
}

#[Object]
impl Query {
    async fn string(&self, input: String) -> String {
        input
    }

    async fn int(&self, input: u32) -> u32 {
        input
    }

    async fn float(&self, input: f32) -> f32 {
        input
    }

    async fn id(&self, input: ID) -> ID {
        input
    }

    async fn list_of_strings(&self, input: Vec<String>) -> Vec<String> {
        input
    }

    async fn list_of_list_of_strings(&self, input: Vec<Vec<String>>) -> Vec<Vec<String>> {
        input
    }

    async fn optional_list_of_optional_strings(
        &self,
        input: Option<Vec<Option<String>>>,
    ) -> Option<Vec<Option<String>>> {
        input
    }

    async fn input_object(&self, input: InputObj) -> Option<Json<InputObj>> {
        Some(Json(input))
    }

    async fn list_of_input_object(&self, input: InputObj) -> Json<InputObj> {
        Json(input)
    }

    async fn fancy_bool(&self, input: FancyBool) -> FancyBool {
        input
    }

    async fn response_header(&self, name: String, value: String) -> Option<bool> {
        self.response_headers.push((name, value));
        None
    }

    async fn header(&self, name: String) -> Option<String> {
        self.headers
            .iter()
            .find(|(key, _)| key == &name)
            .map(|(_, value)| value.clone())
    }

    async fn headers(&self) -> Vec<Header> {
        let mut headers = self
            .headers
            .clone()
            .into_iter()
            .filter(|(name, _)| name != "host")
            .map(|(name, value)| Header { name, value })
            .collect::<Vec<_>>();
        headers.sort_unstable();
        headers
    }
}

#[derive(SimpleObject, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Header {
    name: String,
    value: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Enum, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum FancyBool {
    Yes,
    No,
}

#[derive(InputObject, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InputObj {
    #[serde(skip_serializing_if = "MaybeUndefined::is_undefined")]
    string: MaybeUndefined<String>,
    #[serde(skip_serializing_if = "MaybeUndefined::is_undefined")]
    int: MaybeUndefined<u32>,
    #[serde(skip_serializing_if = "MaybeUndefined::is_undefined")]
    float: MaybeUndefined<f32>,
    #[serde(skip_serializing_if = "MaybeUndefined::is_undefined")]
    id: MaybeUndefined<ID>,
    #[serde(skip_serializing_if = "MaybeUndefined::is_undefined")]
    annoyingly_optional_strings: MaybeUndefined<Vec<Option<Vec<Option<String>>>>>,
    #[serde(skip_serializing_if = "MaybeUndefined::is_undefined")]
    recursive_object: MaybeUndefined<Box<InputObj>>,
    #[serde(skip_serializing_if = "MaybeUndefined::is_undefined")]
    recursive_object_list: MaybeUndefined<Vec<InputObj>>,
    #[serde(skip_serializing_if = "MaybeUndefined::is_undefined")]
    fancy_bool: MaybeUndefined<FancyBool>,
}

mod builder;

use std::{collections::HashMap, future::IntoFuture, sync::Arc};

use engine::{Request, RequestExtensions, RequestHeaders, Response, Schema, StreamingPayload, Variables};
use futures::{future::BoxFuture, Stream, StreamExt};
use serde::Deserialize;

pub use self::builder::{EngineBuilder, RequestContext};

/// An instance of the grafbase-engine code that can be used for testing.
#[derive(Clone)]
pub struct Engine {
    inner: Arc<Inner>,
}

struct Inner {
    schema: Schema,
}

impl Engine {
    pub async fn new(schema: String) -> Self {
        EngineBuilder::new(schema).build().await
    }

    pub fn execute(&self, operation: impl Into<GraphQlRequest>) -> ExecutionRequest<'_> {
        ExecutionRequest {
            graphql: operation.into(),
            headers: HashMap::new(),
            schema: &self.inner.schema,
        }
    }

    pub fn execute_stream(&self, operation: impl Into<GraphQlRequest>) -> StreamExecutionRequest<'_> {
        StreamExecutionRequest {
            graphql: operation.into(),
            headers: HashMap::new(),
            schema: &self.inner.schema,
        }
    }
}

#[must_use]
pub struct ExecutionRequest<'a> {
    graphql: GraphQlRequest,
    headers: HashMap<String, String>,
    schema: &'a Schema,
}

impl ExecutionRequest<'_> {
    /// Adds a header into the request
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn variables(mut self, variables: impl serde::Serialize) -> Self {
        self.graphql.variables = Some(Variables::from_json(
            serde_json::to_value(variables).expect("variables to be serializable"),
        ));
        self
    }
}

impl<'a> IntoFuture for ExecutionRequest<'a> {
    type Output = Response;

    type IntoFuture = BoxFuture<'a, Response>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let request = self
                .graphql
                .into_engine_request()
                .data(RequestHeaders::from(&self.headers));

            self.schema.execute(request).await
        })
    }
}

#[must_use]
pub struct StreamExecutionRequest<'a> {
    graphql: GraphQlRequest,
    headers: HashMap<String, String>,
    schema: &'a Schema,
}

impl<'a> StreamExecutionRequest<'a> {
    /// Adds a header into the request
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Converts the execution request into a Stream
    pub fn into_stream(self) -> impl Stream<Item = StreamingPayload> + 'a {
        let request = self
            .graphql
            .into_engine_request()
            .data(RequestHeaders::from(&self.headers));

        self.schema.execute_stream(request)
    }

    // Collects the StreamingPayloads into a vec
    pub async fn collect(self) -> Vec<StreamingPayload> {
        self.into_stream().collect().await
    }

    /// Converts the execution request into an iterator
    pub async fn into_iter(self) -> impl Iterator<Item = StreamingPayload> {
        self.collect().await.into_iter()
    }
}

#[derive(serde::Serialize, Default)]
pub struct GraphQlRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Variables>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<RequestExtensions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_id: Option<String>,
}

impl GraphQlRequest {
    pub fn into_engine_request(self) -> engine::Request {
        let mut request = Request::new(self.query.unwrap_or_default());
        if let Some(name) = self.operation_name {
            request = request.with_operation_name(name);
        }
        if let Some(variables) = self.variables {
            request = request.variables(variables);
        }
        if let Some(extensions) = self.extensions {
            request.extensions = extensions;
        }
        request.operation_plan_cache_key.doc_id = self.doc_id;
        request
    }
}

impl From<&str> for GraphQlRequest {
    fn from(val: &str) -> Self {
        GraphQlRequest {
            query: Some(val.into()),
            operation_name: None,
            variables: None,
            extensions: None,
            doc_id: None,
        }
    }
}

impl From<String> for GraphQlRequest {
    fn from(val: String) -> Self {
        GraphQlRequest {
            query: Some(val),
            operation_name: None,
            variables: None,
            extensions: None,
            doc_id: None,
        }
    }
}

impl<T, V> From<cynic::Operation<T, V>> for GraphQlRequest
where
    V: serde::Serialize,
{
    fn from(operation: cynic::Operation<T, V>) -> Self {
        GraphQlRequest {
            query: Some(operation.query),
            variables: Some(serde_json::from_value(serde_json::to_value(operation.variables).unwrap()).unwrap()),
            operation_name: operation.operation_name.map(|name| name.to_string()),
            extensions: None,
            doc_id: None,
        }
    }
}

#[async_trait::async_trait]
impl graphql_mocks::Schema for Engine {
    async fn execute(
        &self,
        headers: Vec<(String, String)>,
        request: async_graphql::Request,
    ) -> async_graphql::Response {
        let operation = GraphQlRequest {
            query: Some(request.query),
            operation_name: request.operation_name,
            variables: Some(engine::Variables::deserialize(serde_json::to_value(request.variables).unwrap()).unwrap()),
            extensions: None,
            doc_id: None,
        };

        let mut request = self.execute(operation);
        for (name, value) in headers {
            request = request.header(name, value);
        }

        let response = request.await;

        // Not sure this will work but lets see
        async_graphql::Response::deserialize(serde_json::to_value(response).unwrap()).unwrap()
    }

    fn execute_stream(
        &self,
        _request: async_graphql::Request,
    ) -> futures::stream::BoxStream<'static, async_graphql::Response> {
        todo!("if you need this you should implement it")
    }

    fn sdl(&self) -> String {
        self.inner.schema.federation_sdl()
    }
}

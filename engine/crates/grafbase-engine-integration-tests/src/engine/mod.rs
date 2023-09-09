mod builder;

use std::{collections::HashMap, future::IntoFuture, sync::Arc};

use futures::{future::BoxFuture, Stream, StreamExt};
use grafbase_engine::{Request, RequestHeaders, Response, Schema, StreamingPayload, Variables};

pub use self::builder::EngineBuilder;

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
}

impl<'a> IntoFuture for ExecutionRequest<'a> {
    type Output = Response;

    type IntoFuture = BoxFuture<'a, Response>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let mut request = Request::new(self.graphql.query).data(RequestHeaders::from(&self.headers));
            if let Some(name) = self.graphql.operation_name {
                request = request.operation_name(name);
            }
            if let Some(variables) = self.graphql.variables {
                request = request.variables(variables);
            }
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
        let mut request = Request::new(self.graphql.query).data(RequestHeaders::from(&self.headers));
        if let Some(name) = self.graphql.operation_name {
            request = request.operation_name(name);
        }
        if let Some(variables) = self.graphql.variables {
            request = request.variables(variables);
        }
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

pub struct GraphQlRequest {
    query: String,
    operation_name: Option<String>,
    variables: Option<Variables>,
}

impl From<&str> for GraphQlRequest {
    fn from(val: &str) -> Self {
        GraphQlRequest {
            query: val.into(),
            operation_name: None,
            variables: None,
        }
    }
}

impl From<String> for GraphQlRequest {
    fn from(val: String) -> Self {
        GraphQlRequest {
            query: val,
            operation_name: None,
            variables: None,
        }
    }
}

impl<T, V> From<cynic::Operation<T, V>> for GraphQlRequest
where
    V: serde::Serialize,
{
    fn from(operation: cynic::Operation<T, V>) -> Self {
        GraphQlRequest {
            query: operation.query,
            variables: Some(serde_json::from_value(serde_json::to_value(operation.variables).unwrap()).unwrap()),
            operation_name: operation.operation_name.map(|name| name.to_string()),
        }
    }
}

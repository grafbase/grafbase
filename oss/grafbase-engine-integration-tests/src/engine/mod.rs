mod builder;

use std::{collections::HashMap, future::IntoFuture, sync::Arc};

use futures::future::BoxFuture;
use grafbase_engine::{Request, RequestHeaders, Response, Schema, Variables};

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

mod builder;

use std::{collections::HashMap, future::IntoFuture};

pub use builder::*;
use engine::{ServerResult, Variables};
use futures::future::BoxFuture;

use crate::engine::GraphQlRequest;

pub struct TestFederationEngine {
    engine: engine_v2::Engine,
}

impl TestFederationEngine {
    pub fn execute(&self, operation: impl Into<GraphQlRequest>) -> ExecutionRequest<'_> {
        ExecutionRequest {
            graphql: operation.into(),
            headers: HashMap::new(),
            engine: &self.engine,
        }
    }
}

#[must_use]
pub struct ExecutionRequest<'a> {
    graphql: GraphQlRequest,
    #[allow(dead_code)]
    headers: HashMap<String, String>,
    engine: &'a engine_v2::Engine,
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
    type Output = ServerResult<serde_json::Value>;

    type IntoFuture = BoxFuture<'a, ServerResult<serde_json::Value>>;

    fn into_future(self) -> Self::IntoFuture {
        let document = engine_parser::parse_query(self.graphql.query).expect("request document to be well formed");

        let mut operations = document.operations.iter();

        let operation = match self.graphql.operation_name {
            None => operations.next().expect("document to have at least one operation"),
            Some(expected_name) => operations
                .find(|(name, _)| *name.expect("names if operationName provided") == expected_name)
                .expect("an operation with the given operationName"),
        }
        .1
        .clone()
        .node;

        Box::pin(async move { self.engine.execute(operation).await })
    }
}

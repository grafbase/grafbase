mod builder;

use std::{borrow::Cow, collections::HashMap, future::IntoFuture, ops::Deref};

pub use builder::*;
use engine::Variables;
use futures::{future::BoxFuture, Stream, StreamExt};

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
    type Output = GraphqlResponse;

    type IntoFuture = BoxFuture<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let request = self.graphql.into_engine_request();

        Box::pin(async move {
            let response = self.engine.execute(request, (&self.headers).into()).await;

            GraphqlResponse {
                gql_response: serde_json::to_value(&response).unwrap(),
                metadata: response.take_metadata(),
            }
        })
    }
}

impl<'a> ExecutionRequest<'a> {
    pub fn into_stream(self) -> impl Stream<Item = GraphqlResponse> + 'a {
        let request = self.graphql.into_engine_request();

        self.engine
            .execute_stream(request, (&self.headers).into())
            .map(|response| GraphqlResponse {
                gql_response: serde_json::to_value(&response).unwrap(),
                metadata: response.take_metadata(),
            })
    }
}

#[derive(serde::Serialize, Debug)]
pub struct GraphqlResponse {
    #[serde(flatten)]
    gql_response: serde_json::Value,
    #[serde(skip)]
    pub metadata: engine_v2::ExecutionMetadata,
}

impl std::fmt::Display for GraphqlResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self.gql_response).unwrap())
    }
}

impl Deref for GraphqlResponse {
    type Target = serde_json::Value;

    fn deref(&self) -> &Self::Target {
        &self.gql_response
    }
}

impl GraphqlResponse {
    pub fn into_value(self) -> serde_json::Value {
        self.gql_response
    }

    pub fn into_data(self) -> serde_json::Value {
        assert!(self.errors().is_empty(), "{self:#?}");

        match self.gql_response {
            serde_json::Value::Object(mut value) => value.remove("data"),
            _ => None,
        }
        .unwrap_or_default()
    }

    pub fn errors(&self) -> Cow<'_, Vec<serde_json::Value>> {
        self.gql_response["errors"]
            .as_array()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(Vec::new()))
    }
}

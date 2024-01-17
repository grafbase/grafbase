mod builder;

use std::{borrow::Cow, collections::HashMap, future::IntoFuture, ops::Deref, sync::Arc};

use async_runtime::stream::StreamExt as _;
pub use builder::*;
use engine::Variables;
use futures::{future::BoxFuture, SinkExt, Stream, StreamExt};
use gateway_core::RequestContext as _;
use http::HeaderMap;

use crate::engine::{GraphQlRequest, RequestContext};

pub struct TestFederationGateway {
    gateway: Arc<gateway_v2::Gateway>,
}

impl TestFederationGateway {
    pub fn execute(&self, operation: impl Into<GraphQlRequest>) -> ExecutionRequest {
        ExecutionRequest {
            graphql: operation.into(),
            headers: HashMap::new(),
            gateway: Arc::clone(&self.gateway),
        }
    }
}

#[must_use]
pub struct ExecutionRequest {
    graphql: GraphQlRequest,
    #[allow(dead_code)]
    headers: HashMap<String, String>,
    gateway: Arc<gateway_v2::Gateway>,
}

impl ExecutionRequest {
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

impl IntoFuture for ExecutionRequest {
    type Output = GraphqlResponse;

    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let request = self.graphql.into_engine_request();

        let (ctx, futures) = RequestContext::new(self.headers);
        Box::pin(async move {
            let session = match self.gateway.authorize(ctx.headers_as_map().into()).await {
                Ok(session) => session,
                Err(response) => {
                    return GraphqlResponse {
                        gql_response: serde_json::to_value(&response).unwrap(),
                        metadata: Default::default(),
                        headers: HeaderMap::new(),
                    }
                }
            };

            let response = session.execute(&ctx, request).await;
            tokio::spawn(RequestContext::wait_for_all(futures));

            GraphqlResponse {
                gql_response: serde_json::from_slice(&response.bytes).unwrap(),
                metadata: response.metadata,
                headers: response.headers,
            }
        })
    }
}

impl ExecutionRequest {
    pub fn into_stream(self) -> impl Stream<Item = GraphqlResponse> {
        let request = self.graphql.into_engine_request();

        let (mut sender, receiver) = futures::channel::mpsc::channel(4);

        receiver.join(async move {
            let session = match self.gateway.authorize(self.headers.into()).await {
                Ok(session) => session,
                Err(error) => {
                    sender.send(error.into()).await.ok();
                    return;
                }
            };

            session
                .execute_stream(request)
                .map(|response| Ok(response.into()))
                .forward(sender)
                .await
                .ok();
        })
    }
}

#[derive(serde::Serialize, Debug)]
pub struct GraphqlResponse {
    #[serde(flatten)]
    gql_response: serde_json::Value,
    #[serde(skip)]
    pub metadata: engine_v2::ExecutionMetadata,
    #[serde(skip)]
    pub headers: http::HeaderMap,
}

impl From<engine_v2::Response> for GraphqlResponse {
    fn from(value: engine_v2::Response) -> Self {
        GraphqlResponse {
            metadata: value.metadata().clone(),
            gql_response: serde_json::to_value(value).unwrap(),
            headers: Default::default(),
        }
    }
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

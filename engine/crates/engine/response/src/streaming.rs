use grafbase_tracing::gql_response_status::GraphqlResponseStatus;
use graph_entities::QueryResponse;
use query_path::QueryPath;
use serde::{ser::SerializeMap, Serialize};

use crate::{error::ServerError, GraphQlResponse, Response};

/// If a user makes a streaming request, this is the set of different response payloads
/// they can received.  The first payload will always be an `InitialResponse` - followed by
/// zero or more `Incremental` payloads (if there were any deferred workloads in the request).
///
/// At some point we might add support for subscriptions in which case a user will probably
/// see multiple Response entries.
#[derive(Debug)]
pub enum StreamingPayload {
    InitialResponse(InitialResponse),
    Incremental(IncrementalPayload),
}

impl StreamingPayload {
    pub fn status(&self) -> GraphqlResponseStatus {
        match self {
            StreamingPayload::InitialResponse(InitialResponse { response, .. }) => response.status(),
            StreamingPayload::Incremental(IncrementalPayload { errors, .. }) => {
                if errors.is_empty() {
                    GraphqlResponseStatus::Success
                } else {
                    GraphqlResponseStatus::FieldError {
                        count: errors.len() as u64,
                        // Couldn't have an incremental response otherwise.
                        data_is_null: false,
                    }
                }
            }
        }
    }
}

/// The initial streaming response is _almost_ identical to a standard response, but with the
/// `hasNext` key in it.
#[derive(Debug)]
pub struct InitialResponse {
    /// The standard GraphQL response data
    pub response: Response,

    /// Whether the client should expect more data or not.
    pub has_next: bool,
}

impl serde::Serialize for StreamingPayload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            StreamingPayload::InitialResponse(InitialResponse { response, has_next }) => {
                #[derive(Serialize)]
                struct Serialized<'a> {
                    #[serde(flatten)]
                    response: GraphQlResponse<'a>,
                    #[serde(rename = "hasNext")]
                    has_next: bool,
                }

                Serialized {
                    response: response.to_graphql_response(),
                    has_next: *has_next,
                }
                .serialize(serializer)
            }
            StreamingPayload::Incremental(incremental) => incremental.to_graphql_response().serialize(serializer),
        }
    }
}

/// An incremental response payload as described in the [stream & defer RFC][1].
///
/// This is very similar to the main Response payload, but with additional fields for
/// `label`, `path` & `has_next`.
///
/// [1]: https://github.com/graphql/graphql-wg/blob/main/rfcs/DeferStream.md#payload-format
#[derive(Debug, Default)]
pub struct IncrementalPayload {
    pub label: Option<String>,
    pub data: QueryResponse,
    pub path: QueryPath,
    pub has_next: bool,
    pub errors: Vec<ServerError>,
}

impl Response {
    pub fn into_streaming_payload(self, has_next: bool) -> StreamingPayload {
        StreamingPayload::InitialResponse(InitialResponse {
            response: self,
            has_next,
        })
    }
}

impl From<IncrementalPayload> for StreamingPayload {
    fn from(val: IncrementalPayload) -> Self {
        StreamingPayload::Incremental(val)
    }
}

impl IncrementalPayload {
    pub fn to_graphql_response(&self) -> GraphqlIncrementalPayload<'_> {
        GraphqlIncrementalPayload(self)
    }
}

/// A wrapper around IncrementalPayload that Serialises in GraphQL format
pub struct GraphqlIncrementalPayload<'a>(&'a IncrementalPayload);

impl serde::Serialize for GraphqlIncrementalPayload<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // This is almost exactly what a derive could generate but:
        // 1. It's for the structure nested inside.
        // 2. It calls `self.0.data.as_graphql_data()`
        let mut map = serializer.serialize_map(Some(5))?;
        map.serialize_entry("data", &self.0.data.as_graphql_data())?;
        map.serialize_entry("path", &self.0.path.iter().collect::<Vec<_>>())?;
        map.serialize_entry("hasNext", &self.0.has_next)?;
        if let Some(label) = &self.0.label {
            map.serialize_entry("label", &label)?;
        }
        if !self.0.errors.is_empty() {
            map.serialize_entry("errors", &self.0.errors)?;
        }
        map.end()
    }
}

use grafbase_tracing::gql_response_status::GraphqlResponseStatus;
use graph_entities::CompactValue;
use query_path::QueryPath;

use crate::{error::ServerError, Response};

/// If a user makes a streaming request, this is the set of different response payloads
/// they can received.  The first payload will always be an `InitialResponse` - followed by
/// zero or more `Incremental` payloads (if there were any deferred workloads in the request).
///
/// At some point we might add support for subscriptions in which case a user will probably
/// see multiple Response entries.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum StreamingPayload {
    Incremental(IncrementalPayload),
    InitialResponse(InitialResponse),
}

impl StreamingPayload {
    pub fn status(&self) -> GraphqlResponseStatus {
        match self {
            StreamingPayload::InitialResponse(InitialResponse { data, errors, .. }) => {
                if errors.is_empty() {
                    GraphqlResponseStatus::Success
                } else if data.is_none() {
                    GraphqlResponseStatus::RequestError {
                        count: errors.len() as u64,
                    }
                } else {
                    GraphqlResponseStatus::FieldError {
                        count: errors.len() as u64,
                        data_is_null: data.as_ref().unwrap().is_null(),
                    }
                }
            }
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
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitialResponse {
    /// The standard GraphQL response data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<CompactValue>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ServerError>,

    /// Whether the client should expect more data or not.
    pub has_next: bool,
}

impl InitialResponse {
    pub fn error(response: Response) -> Self {
        InitialResponse {
            data: response.data.into_compact_value(),
            errors: response.errors,
            has_next: false,
        }
    }
}

/// An incremental response payload as described in the [stream & defer RFC][1].
///
/// This is very similar to the main Response payload, but with additional fields for
/// `label`, `path` & `has_next`.
///
/// [1]: https://github.com/graphql/graphql-wg/blob/main/rfcs/DeferStream.md#payload-format
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalPayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub data: CompactValue,
    pub path: QueryPath,
    pub has_next: bool,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ServerError>,
}

impl Response {
    pub fn into_streaming_payload(self, has_next: bool) -> StreamingPayload {
        StreamingPayload::InitialResponse(InitialResponse {
            data: self.data.into_compact_value(),
            errors: self.errors,
            has_next,
        })
    }
}

impl From<IncrementalPayload> for StreamingPayload {
    fn from(val: IncrementalPayload) -> Self {
        StreamingPayload::Incremental(val)
    }
}

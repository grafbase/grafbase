use std::borrow::Cow;

use engine::{ErrorResponse, GraphqlError};
use http::{
    HeaderValue,
    header::{CONTENT_LENGTH, CONTENT_TYPE},
};

#[derive(serde::Serialize)]
struct ErrorResponseWrapper {
    errors: Vec<GraphqlErrorWrapper>,
}

#[derive(serde::Serialize)]
struct GraphqlErrorWrapper {
    message: Cow<'static, str>,
    extensions: Vec<(Cow<'static, str>, serde_json::Value)>,
}

impl From<ErrorResponse> for ErrorResponseWrapper {
    fn from(value: ErrorResponse) -> Self {
        Self {
            errors: value.errors.into_iter().map(GraphqlErrorWrapper::from).collect(),
        }
    }
}

impl From<GraphqlError> for GraphqlErrorWrapper {
    fn from(value: GraphqlError) -> Self {
        Self {
            message: value.message,
            extensions: value.extensions,
        }
    }
}

/// Converts an ErrorResponse into an HTTP Response, respecting the Accept header
pub(crate) fn error_response_to_http<B>(
    response_format: ResponseFormat,
    error_response: ErrorResponse,
) -> http::Response<B>
where
    B: From<Vec<u8>>,
{
    let status = error_response.status;
    let error_response = ErrorResponseWrapper::from(error_response);
    let bytes = sonic_rs::to_vec(&error_response).unwrap();

    // Build response with appropriate content type
    http::Response::builder()
        .status(status)
        .header(CONTENT_TYPE, response_format.content_type())
        .header(CONTENT_LENGTH, bytes.len())
        .body(B::from(bytes))
        .expect("building error response should not fail")
}

#[derive(Clone, Copy)]
pub(crate) enum ResponseFormat {
    Json,
    GraphqlResponseJson,
}

impl ResponseFormat {
    fn content_type(self) -> HeaderValue {
        match self {
            ResponseFormat::Json => HeaderValue::from_static("application/json"),
            ResponseFormat::GraphqlResponseJson => HeaderValue::from_static("application/graphql-response+json"),
        }
    }
}

pub(crate) fn extract_response_format(headers: &http::HeaderMap) -> ResponseFormat {
    if headers
        .get_all("accept")
        .iter()
        .filter_map(|value| value.to_str().ok())
        .any(|value| value.contains("application/graphql-response+json"))
    {
        ResponseFormat::GraphqlResponseJson
    } else {
        ResponseFormat::Json
    }
}

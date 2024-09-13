use itertools::Itertools;

use crate::{
    graphql_over_http::ResponseFormat,
    response::{ErrorCode, GraphqlError, Response},
};

impl Response {
    pub(crate) fn not_acceptable_error() -> Response {
        let message = format!(
            "Missing or invalid Accept header. You must specify one of: {}.",
            ResponseFormat::supported_media_types()
                .iter()
                .format_with(", ", |media_type, f| { f(&format_args!("'{}'", media_type)) }),
        );
        Response::refuse_request_with(
            http::StatusCode::NOT_ACCEPTABLE,
            GraphqlError::new(message, ErrorCode::BadRequest),
        )
    }

    pub(crate) fn unsupported_media_type() -> Response {
        Response::refuse_request_with(
            http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
            GraphqlError::new(
                "Missing or invalid Content-Type header. Only 'application/json' is supported.",
                ErrorCode::BadRequest,
            ),
        )
    }

    pub(crate) fn method_not_allowed(message: &'static str) -> Response {
        Response::refuse_request_with(
            http::StatusCode::METHOD_NOT_ALLOWED,
            GraphqlError::new(message, ErrorCode::BadRequest),
        )
    }

    pub(crate) fn gateway_rate_limited() -> Response {
        Response::refuse_request_with(
            http::StatusCode::TOO_MANY_REQUESTS,
            GraphqlError::new("Rate limited", ErrorCode::RateLimited),
        )
    }

    pub(crate) fn unauthenticated() -> Response {
        Response::refuse_request_with(
            http::StatusCode::UNAUTHORIZED,
            GraphqlError::new("Unauthenticated", ErrorCode::Unauthenticated),
        )
    }

    // https://github.com/graphql/graphql-over-http/blob/main/spec/GraphQLOverHTTP.md
    pub(crate) fn not_well_formed_graphql_over_http_request(message: impl std::fmt::Display) -> Response {
        Response::refuse_request_with(
            http::StatusCode::BAD_REQUEST,
            GraphqlError::new(
                format!("Bad request, GraphQL request is not well formed: {message}"),
                ErrorCode::BadRequest,
            ),
        )
    }
    // We assume any invalid request error would be raised before the timeout expires. So if we do
    // end up sending this error it means operation was valid and the query was just slow.
    pub(crate) fn gateway_timeout() -> Response {
        Response::request_error(None, [GraphqlError::new("Gateway timeout", ErrorCode::GatewayTimeout)])
    }

    pub(crate) fn bad_request_but_well_formed_graphql_over_http_request(message: &str) -> Response {
        Response::request_error(
            None,
            [GraphqlError::new(
                format!("Bad request: {message}"),
                ErrorCode::BadRequest,
            )],
        )
    }
}

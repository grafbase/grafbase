use std::borrow::Cow;

use gateway_config::ErrorCodeMapping;
use itertools::Itertools;

use crate::{
    Body,
    graphql_over_http::{ContentType, Http, ResponseFormat},
    response::{ErrorCode, GraphqlError, Response},
};

pub(crate) fn not_acceptable_error(
    error_code_mapping: ErrorCodeMapping,
    format: ResponseFormat,
) -> http::Response<Body> {
    let message = format!(
        "Missing or invalid Accept header. You must specify one of: {}.",
        ResponseFormat::supported_media_types()
            .iter()
            .format_with(", ", |media_type, f| { f(&format_args!("'{media_type}'")) }),
    );
    Http::error(
        format,
        refuse_request_with(error_code_mapping, http::StatusCode::NOT_ACCEPTABLE, message),
    )
}

pub(crate) fn unsupported_content_type(
    error_code_mapping: ErrorCodeMapping,
    format: ResponseFormat,
) -> http::Response<Body> {
    Http::error(
        format,
        refuse_request_with(
            error_code_mapping,
            http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
            format!(
                "Missing or invalid Content-Type header. You must specify one of: {}",
                ContentType::supported()
                    .iter()
                    .format_with(", ", |media_type, f| f(&format_args!(
                        "'{}'",
                        media_type.to_str().unwrap()
                    ))),
            ),
        ),
    )
}

pub(crate) fn method_not_allowed(
    error_code_mapping: ErrorCodeMapping,
    format: ResponseFormat,
    message: &'static str,
) -> http::Response<Body> {
    Http::error(
        format,
        refuse_request_with(error_code_mapping, http::StatusCode::METHOD_NOT_ALLOWED, message),
    )
}

// https://github.com/graphql/graphql-over-http/blob/main/spec/GraphQLOverHTTP.md
pub(crate) fn not_well_formed_graphql_over_http_request(
    error_code_mapping: ErrorCodeMapping,
    message: impl std::fmt::Display,
) -> Response {
    refuse_request_with(
        error_code_mapping,
        http::StatusCode::BAD_REQUEST,
        format!("Bad request: GraphQL request is not well formed: {message}"),
    )
}

pub(crate) fn refuse_request_with(
    error_code_mapping: ErrorCodeMapping,
    status_code: http::StatusCode,
    message: impl Into<Cow<'static, str>>,
) -> Response {
    Response::refused_request(
        error_code_mapping,
        status_code,
        [GraphqlError::new(message, ErrorCode::BadRequest)],
        Default::default(),
    )
}

pub(crate) mod response {
    use gateway_config::ErrorCodeMapping;

    use crate::{
        ErrorCode,
        response::{GraphqlError, Response},
    };

    pub(crate) fn gateway_rate_limited(error_code_mapping: ErrorCodeMapping) -> Response {
        Response::refused_request(
            error_code_mapping,
            http::StatusCode::TOO_MANY_REQUESTS,
            [GraphqlError::new("Rate limited", ErrorCode::RateLimited)],
            Default::default(),
        )
    }

    // We assume any invalid request error would be raised before the timeout expires. So if we do
    // end up sending this error it means operation was valid and the query was just slow.
    pub(crate) fn gateway_timeout(error_code_mapping: ErrorCodeMapping) -> Response {
        let error = GraphqlError::new("Gateway timeout", ErrorCode::GatewayTimeout);
        Response::request_error(error_code_mapping, [error])
    }
}

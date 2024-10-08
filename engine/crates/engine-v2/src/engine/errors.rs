use std::borrow::Cow;

use itertools::Itertools;

use crate::{
    graphql_over_http::{Http, ResponseFormat},
    response::{ErrorCode, GraphqlError, Response},
    Body,
};

pub(crate) fn not_acceptable_error(format: ResponseFormat) -> http::Response<Body> {
    let message = format!(
        "Missing or invalid Accept header. You must specify one of: {}.",
        ResponseFormat::supported_media_types()
            .iter()
            .format_with(", ", |media_type, f| { f(&format_args!("'{}'", media_type)) }),
    );
    refuse_request_with(format, http::StatusCode::NOT_ACCEPTABLE, message)
}

pub(crate) fn unsupported_media_type(format: ResponseFormat) -> http::Response<Body> {
    refuse_request_with(
        format,
        http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "Missing or invalid Content-Type header. Only 'application/json' is supported.",
    )
}

pub(crate) fn method_not_allowed(format: ResponseFormat, message: &'static str) -> http::Response<Body> {
    refuse_request_with(format, http::StatusCode::METHOD_NOT_ALLOWED, message)
}

// https://github.com/graphql/graphql-over-http/blob/main/spec/GraphQLOverHTTP.md
pub(crate) fn not_well_formed_graphql_over_http_request(
    format: ResponseFormat,
    message: impl std::fmt::Display,
) -> http::Response<Body> {
    refuse_request_with(
        format,
        http::StatusCode::BAD_REQUEST,
        format!("Bad request: GraphQL request is not well formed: {message}"),
    )
}

pub(crate) fn refuse_request_with(
    format: ResponseFormat,
    status_code: http::StatusCode,
    message: impl Into<Cow<'static, str>>,
) -> http::Response<Body> {
    Http::error(
        format,
        Response::<()>::refuse_request_with(status_code, GraphqlError::new(message, ErrorCode::BadRequest)),
    )
}

pub(crate) fn bad_request_but_well_formed_graphql_over_http_request(
    format: ResponseFormat,
    message: impl std::fmt::Display,
) -> http::Response<Body> {
    Http::error(
        format,
        Response::<()>::request_error(
            None,
            [GraphqlError::new(
                format!("Bad request: {message}"),
                ErrorCode::BadRequest,
            )],
        ),
    )
}

pub(crate) fn gateway_timeout(format: ResponseFormat) -> http::Response<Body> {
    Http::error(format, self::response::gateway_timeout::<()>())
}

pub(crate) mod response {
    use crate::{
        response::{GraphqlError, Response},
        ErrorCode,
    };

    pub(crate) fn mutation_not_allowed_with_safe_method<OnOperationResponseHookOutput>(
    ) -> Response<OnOperationResponseHookOutput> {
        Response::refuse_request_with(
            http::StatusCode::METHOD_NOT_ALLOWED,
            GraphqlError::new(
                "Mutation is not allowed with a safe method like GET",
                ErrorCode::BadRequest,
            ),
        )
    }

    pub(crate) fn gateway_rate_limited<OnOperationResponseHookOutput>() -> Response<OnOperationResponseHookOutput> {
        Response::refuse_request_with(
            http::StatusCode::TOO_MANY_REQUESTS,
            GraphqlError::new("Rate limited", ErrorCode::RateLimited),
        )
    }

    pub(crate) fn unauthenticated<OnOperationResponseHookOutput>() -> Response<OnOperationResponseHookOutput> {
        Response::refuse_request_with(
            http::StatusCode::UNAUTHORIZED,
            GraphqlError::new("Unauthenticated", ErrorCode::Unauthenticated),
        )
    }

    // We assume any invalid request error would be raised before the timeout expires. So if we do
    // end up sending this error it means operation was valid and the query was just slow.
    pub(crate) fn gateway_timeout<OnOperationResponseHookOutput>() -> Response<OnOperationResponseHookOutput> {
        Response::request_error(None, [GraphqlError::new("Gateway timeout", ErrorCode::GatewayTimeout)])
    }
}

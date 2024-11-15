use std::borrow::Cow;

use itertools::Itertools;

use crate::{
    graphql_over_http::{Http, ResponseFormat},
    response::{ErrorCode, GraphqlError, Response},
    Body,
};

use super::{Engine, RequestContext, Runtime};

pub(crate) fn not_acceptable_error(format: ResponseFormat) -> http::Response<Body> {
    let message = format!(
        "Missing or invalid Accept header. You must specify one of: {}.",
        ResponseFormat::supported_media_types()
            .iter()
            .format_with(", ", |media_type, f| { f(&format_args!("'{}'", media_type)) }),
    );
    Http::error(
        format,
        refuse_request_with::<()>(http::StatusCode::NOT_ACCEPTABLE, message),
    )
}

pub(crate) fn unsupported_media_type(format: ResponseFormat) -> http::Response<Body> {
    Http::error(
        format,
        refuse_request_with::<()>(
            http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Missing or invalid Content-Type header. Only 'application/json' is supported.",
        ),
    )
}

pub(crate) fn method_not_allowed(format: ResponseFormat, message: &'static str) -> http::Response<Body> {
    Http::error(
        format,
        refuse_request_with::<()>(http::StatusCode::METHOD_NOT_ALLOWED, message),
    )
}

// https://github.com/graphql/graphql-over-http/blob/main/spec/GraphQLOverHTTP.md
pub(crate) fn not_well_formed_graphql_over_http_request<OnOperationResponseHookOutput>(
    message: impl std::fmt::Display,
) -> Response<OnOperationResponseHookOutput> {
    refuse_request_with(
        http::StatusCode::BAD_REQUEST,
        format!("Bad request: GraphQL request is not well formed: {message}"),
    )
}

pub(crate) fn refuse_request_with<OnOperationResponseHookOutput>(
    status_code: http::StatusCode,
    message: impl Into<Cow<'static, str>>,
) -> Response<OnOperationResponseHookOutput> {
    Response::<OnOperationResponseHookOutput>::refuse_request_with(
        status_code,
        vec![GraphqlError::new(message, ErrorCode::BadRequest)],
    )
}

impl<R: Runtime> Engine<R> {
    pub(crate) fn bad_request_but_well_formed_graphql_over_http_request(
        &self,
        ctx: &RequestContext,
        message: impl std::fmt::Display,
    ) -> http::Response<Body> {
        Http::error(
            ctx.response_format,
            Response::<()>::request_error(
                None,
                [GraphqlError::new(
                    format!("Bad request: {message}"),
                    ErrorCode::BadRequest,
                )],
            )
            .with_grafbase_extension(self.default_grafbase_response_extension(ctx)),
        )
    }

    pub(crate) fn gateway_timeout_error(&self, ctx: &RequestContext) -> http::Response<Body> {
        Http::error(
            ctx.response_format,
            self::response::gateway_timeout::<()>()
                .with_grafbase_extension(self.default_grafbase_response_extension(ctx)),
        )
    }
}

pub(crate) mod response {
    use crate::{
        response::{GraphqlError, Response},
        ErrorCode,
    };

    pub(crate) fn gateway_rate_limited<OnOperationResponseHookOutput>() -> Response<OnOperationResponseHookOutput> {
        Response::refuse_request_with(
            http::StatusCode::TOO_MANY_REQUESTS,
            vec![GraphqlError::new("Rate limited", ErrorCode::RateLimited)],
        )
    }

    pub(crate) fn unauthenticated<OnOperationResponseHookOutput>() -> Response<OnOperationResponseHookOutput> {
        Response::refuse_request_with(
            http::StatusCode::UNAUTHORIZED,
            vec![GraphqlError::new("Unauthenticated", ErrorCode::Unauthenticated)],
        )
    }

    // We assume any invalid request error would be raised before the timeout expires. So if we do
    // end up sending this error it means operation was valid and the query was just slow.
    pub(crate) fn gateway_timeout<OnOperationResponseHookOutput>() -> Response<OnOperationResponseHookOutput> {
        Response::request_error(None, [GraphqlError::new("Gateway timeout", ErrorCode::GatewayTimeout)])
    }
}

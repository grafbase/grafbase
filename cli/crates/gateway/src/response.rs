use crate::error::Error;
use axum::response::IntoResponse;
use engine::parser::types::OperationType;
use gateway_core::ConstructableResponse;
use http::{header, status::StatusCode, HeaderValue};
use runtime::rate_limiting;
use std::sync::Arc;

pub struct Response {
    inner: axum::response::Response,
}

impl Response {
    pub fn batch_response(responses: Vec<Arc<engine::Response>>) -> Self {
        let body = axum::Json(
            responses
                .iter()
                .map(|response| response.to_graphql_response())
                .collect::<Vec<_>>(),
        );

        body.into_response().into()
    }
}

impl From<crate::Error> for Response {
    fn from(err: crate::Error) -> Self {
        use crate::Error::{BadRequest, Cache, Internal, Serialization};

        match err {
            BadRequest(msg) => Response::error(StatusCode::BAD_REQUEST, &msg),
            Cache(err) => Response::error(StatusCode::INTERNAL_SERVER_ERROR, &err.to_string()),
            Serialization(msg) | Internal(msg) => Response::error(StatusCode::INTERNAL_SERVER_ERROR, &msg),
            Error::Ratelimit(err) => match err {
                rate_limiting::Error::ExceededCapacity => Response::engine(
                    Arc::new(engine::Response::from_errors_with_type(
                        vec![engine::ServerError::new("Too many requests", None)],
                        OperationType::Query,
                    )),
                    Default::default(),
                )
                .unwrap(),
                rate_limiting::Error::Internal(err) => {
                    Response::error(StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
                }
            },
        }
    }
}

impl From<axum::response::Response> for Response {
    fn from(resp: axum::response::Response) -> Self {
        Self { inner: resp }
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        self.inner
    }
}

impl ConstructableResponse for Response {
    type Error = crate::Error;

    fn error(code: StatusCode, message: &str) -> Self {
        println!("ERROR {code} {message}");
        (code, message.to_string()).into_response().into()
    }

    fn engine(response: Arc<engine::Response>, mut headers: http::HeaderMap) -> Result<Self, Self::Error> {
        headers.append(
            header::CONTENT_TYPE,
            HeaderValue::try_from("application/json;charset=UTF-8").unwrap(),
        );
        let body = axum::Json(response.as_ref().to_graphql_response());
        Ok((headers, body).into_response().into())
    }

    fn admin(response: async_graphql::Response) -> Result<Self, Self::Error> {
        Ok(axum::Json(response).into_response().into())
    }
}

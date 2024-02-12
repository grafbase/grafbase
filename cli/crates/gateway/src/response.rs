use axum::response::IntoResponse;
use gateway_core::ConstructableResponse;
use http::{header, status::StatusCode, HeaderValue};
use std::sync::Arc;

pub struct Response {
    inner: axum::response::Response,
}

impl From<crate::Error> for Response {
    fn from(err: crate::Error) -> Self {
        use crate::Error::{BadRequest, Cache, Internal, Serialization};

        match err {
            BadRequest(msg) => Response::error(StatusCode::BAD_REQUEST, &msg),
            Cache(err) => Response::error(StatusCode::INTERNAL_SERVER_ERROR, &err.to_string()),
            Serialization(msg) | Internal(msg) => Response::error(StatusCode::INTERNAL_SERVER_ERROR, &msg),
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

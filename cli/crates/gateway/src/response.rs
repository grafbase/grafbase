use axum::response::IntoResponse;
use http::{header, status::StatusCode};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

pub struct Response {
    inner: axum::response::Response,
}

impl Deref for Response {
    type Target = axum::response::Response;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Response {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl From<Result<Response, crate::Error>> for Response {
    fn from(result: Result<Response, crate::Error>) -> Self {
        match result {
            Ok(resp) => resp,
            Err(err) => err.into(),
        }
    }
}

impl From<crate::Error> for Response {
    fn from(err: crate::Error) -> Self {
        use crate::Error::{BadRequest, Cache, Internal, Serialization};
        use gateway_core::Response;
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

impl gateway_core::Response for Response {
    type Error = crate::Error;

    fn error(code: StatusCode, message: &str) -> Self {
        println!("ERROR {code} {message}");
        (code, message.to_string()).into_response().into()
    }

    fn engine(response: Arc<engine::Response>) -> Result<Self, Self::Error> {
        let headers = [(header::CONTENT_TYPE, "application/json;charset=UTF-8")];
        let body = axum::Json(response.as_ref().to_graphql_response());
        Ok((headers, body).into_response().into())
    }

    fn admin(response: async_graphql::Response) -> Result<Self, Self::Error> {
        Ok(axum::Json(response).into_response().into())
    }

    fn with_additional_headers(mut self, headers: http::HeaderMap) -> Self {
        use std::str::FromStr;
        self.headers_mut().extend(headers.into_iter().map(|(name, value)| {
            (
                name.map(|name| axum::http::HeaderName::from_str(name.as_str()).expect("must be a valid name")),
                axum::http::HeaderValue::from_bytes(value.as_bytes()).expect("must be a valid value"),
            )
        }));
        self
    }
}

use std::future::IntoFuture;

use futures_lite::future::{self, FutureExt};

use crate::{GraphQlRequestBody, GraphqlHttpBatchResponse};

#[must_use]
pub struct TestBatchRequest {
    pub(super) client: reqwest::Client,
    pub(super) parts: http::request::Parts,
    pub(super) body: Vec<GraphQlRequestBody>,
}

impl IntoFuture for TestBatchRequest {
    type Output = GraphqlHttpBatchResponse;

    type IntoFuture = future::Boxed<Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        async move {
            let http::request::Parts {
                method, uri, headers, ..
            } = self.parts;

            let response = self
                .client
                .request(method, uri.to_string())
                .headers(headers)
                .json(&self.body)
                .send()
                .await
                .expect("http request to succeed");

            GraphqlHttpBatchResponse {
                status: response.status(),
                headers: response.headers().clone(),
                body: response.json().await.expect("a json response"),
            }
        }
        .boxed()
    }
}

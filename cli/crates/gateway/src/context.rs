use std::{collections::HashMap, sync::Arc};

use futures_util::future::BoxFuture;
use gateway_core::serving::{AUTHORIZATION_HEADER, X_API_KEY_HEADER};
use http::HeaderMap;
use tokio::sync::mpsc::UnboundedSender;

pub struct Context {
    pub(crate) ray_id: String,
    pub(crate) x_api_key_header: Option<String>,
    pub(crate) authorization_header: Option<String>,
    pub(crate) headers: HeaderMap,
    // TODO: or use a queue?
    wait_until_sender: UnboundedSender<BoxFuture<'static, ()>>,
}

impl Context {
    pub(crate) fn new(
        headers: HeaderMap,
        params: &HashMap<String, String>,
        wait_until_sender: UnboundedSender<BoxFuture<'static, ()>>,
    ) -> Arc<Self> {
        Arc::new(crate::Context {
            ray_id: ulid::Ulid::new().to_string(),
            x_api_key_header: headers
                .get(X_API_KEY_HEADER)
                .and_then(|value| value.to_str().ok().map(ToString::to_string))
                .or_else(|| params.get(X_API_KEY_HEADER).cloned()),
            authorization_header: headers
                .get(AUTHORIZATION_HEADER)
                .and_then(|value| value.to_str().ok().map(ToString::to_string))
                .or_else(|| params.get(AUTHORIZATION_HEADER).cloned()),
            headers,
            wait_until_sender,
        })
    }
}

#[async_trait::async_trait]
impl gateway_core::RequestContext for Context {
    fn ray_id(&self) -> &str {
        &self.ray_id
    }

    async fn wait_until(&self, fut: BoxFuture<'static, ()>) {
        self.wait_until_sender
            .send(fut)
            .ok()
            .expect("Channel is not closed before finishing all wait_until");
    }

    fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}

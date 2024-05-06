use std::{collections::BTreeMap, sync::Arc};

use futures_util::future::BoxFuture;
use http::HeaderMap;
use tokio::sync::mpsc::UnboundedSender;

pub struct Context {
    pub(crate) ray_id: String,
    pub(crate) headers: HeaderMap,
    // TODO: or use a queue?
    pub(crate) wait_until_sender: UnboundedSender<BoxFuture<'static, ()>>,
}

impl Context {
    pub(crate) fn new(headers: HeaderMap, wait_until_sender: UnboundedSender<BoxFuture<'static, ()>>) -> Arc<Self> {
        Arc::new(crate::Context {
            ray_id: ulid::Ulid::new().to_string(),
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
            .expect("Channel is not closed before finishing all wait_until");
    }

    fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}

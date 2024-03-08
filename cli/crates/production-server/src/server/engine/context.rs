use futures_util::future::BoxFuture;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone)]
pub(super) struct RequestContext {
    pub(super) ray_id: String,
    pub(super) headers: http::HeaderMap,
    pub(super) wait_until_sender: UnboundedSender<BoxFuture<'static, ()>>,
}

#[async_trait::async_trait]
impl runtime::context::RequestContext for RequestContext {
    fn ray_id(&self) -> &str {
        &self.ray_id
    }

    async fn wait_until(&self, fut: BoxFuture<'static, ()>) {
        self.wait_until_sender
            .send(fut)
            .expect("Channel is not closed before finishing all wait_until");
    }

    fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }
}

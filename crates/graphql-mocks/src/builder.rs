use std::{future::IntoFuture, sync::Arc};

use futures::{FutureExt, future::BoxFuture};

use crate::{MockGraphQlServer, Schema};

pub struct MockGraphQlServerBuilder {
    schema: Arc<dyn Schema>,
    port: Option<u16>,
}

impl MockGraphQlServerBuilder {
    pub(super) fn new(schema: Arc<dyn Schema>) -> Self {
        MockGraphQlServerBuilder { schema, port: None }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub async fn build(self) -> MockGraphQlServer {
        MockGraphQlServer::new_impl(self.schema, self.port).await
    }
}

impl IntoFuture for MockGraphQlServerBuilder {
    type Output = MockGraphQlServer;

    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        self.build().boxed()
    }
}

use std::pin::Pin;

use http::HeaderMap;
use runtime::user_hooks::{UserHookError, UserHooksImpl};

type GatewayCallback = Pin<Box<dyn Fn(HeaderMap) -> Result<HeaderMap, UserHookError> + Send + Sync + 'static>>;

#[derive(Default)]
pub struct UserHooksTest {
    on_gateway_request: Option<GatewayCallback>,
}

impl UserHooksTest {
    pub fn on_gateway_request<F>(mut self, callback: F) -> Self
    where
        F: Fn(HeaderMap) -> Result<HeaderMap, UserHookError> + Send + Sync + 'static,
    {
        self.on_gateway_request = Some(Box::pin(callback));
        self
    }
}

#[async_trait::async_trait]
impl UserHooksImpl for UserHooksTest {
    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<HeaderMap, UserHookError> {
        match self.on_gateway_request {
            Some(ref callback) => callback(headers),
            None => Ok(headers),
        }
    }
}

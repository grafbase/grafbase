use std::{collections::HashMap, pin::Pin};

use http::HeaderMap;
use runtime::user_hooks::{UserHookError, UserHooksImpl};

type GatewayCallback =
    Pin<Box<dyn Fn(HeaderMap) -> Result<(HashMap<String, String>, HeaderMap), UserHookError> + Send + Sync + 'static>>;

#[derive(Default)]
pub struct UserHooksTest {
    on_gateway_request: Option<GatewayCallback>,
}

impl UserHooksTest {
    pub fn on_gateway_request<F>(mut self, callback: F) -> Self
    where
        F: Fn(HeaderMap) -> Result<(HashMap<String, String>, HeaderMap), UserHookError> + Send + Sync + 'static,
    {
        self.on_gateway_request = Some(Box::pin(callback));
        self
    }
}

#[async_trait::async_trait]
impl UserHooksImpl for UserHooksTest {
    type Context = HashMap<String, String>;

    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), UserHookError> {
        match self.on_gateway_request {
            Some(ref callback) => callback(headers),
            None => Ok((HashMap::new(), headers)),
        }
    }
}

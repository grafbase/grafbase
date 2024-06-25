use std::{collections::HashMap, pin::Pin, sync::Arc};

use http::HeaderMap;
use runtime::hooks::{HookError, HooksImpl, UserError};

type GatewayHook =
    Pin<Box<dyn Fn(HeaderMap) -> Result<(HashMap<String, String>, HeaderMap), HookError> + Send + Sync + 'static>>;

type AuthorizationHook = Pin<
    Box<
        dyn Fn(Arc<HashMap<String, String>>, Vec<String>) -> Result<Vec<Option<UserError>>, HookError>
            + Send
            + Sync
            + 'static,
    >,
>;

#[derive(Default)]
pub struct TestHooks {
    on_gateway_request: Option<GatewayHook>,
    authorized: Option<AuthorizationHook>,
}

impl TestHooks {
    pub fn on_gateway_request<F>(mut self, hook: F) -> Self
    where
        F: Fn(HeaderMap) -> Result<(HashMap<String, String>, HeaderMap), HookError> + Send + Sync + 'static,
    {
        self.on_gateway_request = Some(Box::pin(hook));
        self
    }

    pub fn authorized<F>(mut self, hook: F) -> Self
    where
        F: Fn(Arc<HashMap<String, String>>, Vec<String>) -> Result<Vec<Option<UserError>>, HookError>
            + Send
            + Sync
            + 'static,
    {
        self.authorized = Some(Box::pin(hook));
        self
    }
}

#[async_trait::async_trait]
impl HooksImpl for TestHooks {
    type Context = HashMap<String, String>;

    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), HookError> {
        match self.on_gateway_request {
            Some(ref hook) => hook(headers),
            None => Ok((HashMap::new(), headers)),
        }
    }

    async fn authorized(
        &self,
        context: Arc<Self::Context>,
        input: Vec<String>,
    ) -> Result<Vec<Option<UserError>>, HookError> {
        match self.authorized {
            Some(ref hook) => hook(context, input),
            None => todo!("please define the on-authorization hook before testing"),
        }
    }
}

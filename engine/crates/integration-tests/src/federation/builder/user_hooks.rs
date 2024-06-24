use std::{collections::HashMap, pin::Pin};

use http::HeaderMap;
use runtime::user_hooks::{UserError, UserHookError, UserHooksImpl};

type GatewayHook =
    Pin<Box<dyn Fn(HeaderMap) -> Result<(HashMap<String, String>, HeaderMap), UserHookError> + Send + Sync + 'static>>;

type AuthorizationHook = Pin<
    Box<
        dyn Fn(
                HashMap<String, String>,
                Vec<String>,
            ) -> Result<(HashMap<String, String>, Vec<Option<UserError>>), UserHookError>
            + Send
            + Sync
            + 'static,
    >,
>;

#[derive(Default)]
pub struct UserHooksTest {
    on_gateway_request: Option<GatewayHook>,
    on_authorization: Option<AuthorizationHook>,
}

impl UserHooksTest {
    pub fn on_gateway_request<F>(mut self, hook: F) -> Self
    where
        F: Fn(HeaderMap) -> Result<(HashMap<String, String>, HeaderMap), UserHookError> + Send + Sync + 'static,
    {
        self.on_gateway_request = Some(Box::pin(hook));
        self
    }

    pub fn on_authorization<F>(mut self, hook: F) -> Self
    where
        F: Fn(
                HashMap<String, String>,
                Vec<String>,
            ) -> Result<(HashMap<String, String>, Vec<Option<UserError>>), UserHookError>
            + Send
            + Sync
            + 'static,
    {
        self.on_authorization = Some(Box::pin(hook));
        self
    }
}

#[async_trait::async_trait]
impl UserHooksImpl for UserHooksTest {
    type Context = HashMap<String, String>;

    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), UserHookError> {
        match self.on_gateway_request {
            Some(ref hook) => hook(headers),
            None => Ok((HashMap::new(), headers)),
        }
    }

    async fn on_authorization(
        &self,
        context: Self::Context,
        input: Vec<String>,
    ) -> Result<(Self::Context, Vec<Option<UserError>>), UserHookError> {
        match self.on_authorization {
            Some(ref hook) => hook(context, input),
            None => todo!("please define the on-authorization hook before testing"),
        }
    }
}

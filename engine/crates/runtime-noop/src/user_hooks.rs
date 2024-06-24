use std::collections::HashMap;

use runtime::user_hooks::{HeaderMap, UserHookError, UserHooksImpl};

#[derive(Clone)]
pub struct UserHooksNoop;

#[async_trait::async_trait]
impl UserHooksImpl for UserHooksNoop {
    type Context = HashMap<String, String>;

    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), UserHookError> {
        Ok((HashMap::new(), headers))
    }
}

use runtime::user_hooks::{HeaderMap, UserHookError, UserHooksImpl};

#[derive(Clone)]
pub struct UserHooksNoop;

#[async_trait::async_trait]
impl UserHooksImpl for UserHooksNoop {
    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<HeaderMap, UserHookError> {
        Ok(headers)
    }
}

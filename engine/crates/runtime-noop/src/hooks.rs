use std::collections::HashMap;

use runtime::hooks::{HeaderMap, HookError, HooksImpl, UserError};

#[derive(Clone)]
pub struct HooksNoop;

#[async_trait::async_trait]
impl HooksImpl for HooksNoop {
    type Context = HashMap<String, String>;

    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), HookError> {
        Ok((HashMap::new(), headers))
    }

    async fn on_authorization(
        &self,
        _: Self::Context,
        _: Vec<String>,
    ) -> Result<(Self::Context, Vec<Option<UserError>>), HookError> {
        unreachable!("@authorization directive not available outside of local context")
    }
}

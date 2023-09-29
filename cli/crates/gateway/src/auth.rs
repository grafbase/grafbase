use std::sync::Arc;

use common_types::auth::ExecutionAuth;
use engine::AuthConfig;
use gateway_core::{AdminAuthError, AuthError};
use runtime_local::Bridge;
use runtime_noop::kv::NoopKv;

pub(crate) struct Authorizer {
    pub(crate) auth_config: AuthConfig,
    pub(crate) bridge: Bridge,
}

#[async_trait::async_trait]
impl gateway_core::Authorizer for Authorizer {
    type Context = crate::Context;

    async fn authorize_admin_request(
        &self,
        _ctx: &Arc<Self::Context>,
        _request: &async_graphql::Request,
    ) -> Result<(), AdminAuthError> {
        Ok(())
    }

    async fn authorize_request(
        &self,
        ctx: &Arc<Self::Context>,
        _request: &engine::Request,
    ) -> Result<ExecutionAuth, AuthError> {
        if ctx.x_api_key_header.is_some() {
            Ok(ExecutionAuth::new_from_api_keys())
        } else {
            let auth_invoker = runtime_local::UdfInvokerImpl::new(self.bridge.clone());
            gateway_core::authorize_request(
                &NoopKv,
                &auth_invoker,
                &self.auth_config,
                ctx.as_ref(),
                ctx.authorization_header.clone(),
            )
            .await
        }
    }
}

use std::sync::Arc;

use super::bridge_api;
use crate::LocalContext;

#[derive(Debug, Clone, thiserror::Error)]
pub enum CustomResolversError {
    #[error("Invocation failed")]
    InvocationError,
}

pub struct CustomResolvers {
    local_context: Arc<LocalContext>,
}

impl CustomResolvers {
    pub async fn invoke(
        &self,
        resolver_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, CustomResolversError> {
        bridge_api::invoke_resolver(resolver_name, arguments, &self.local_context.bridge_port)
            .await
            .map_err(|_| CustomResolversError::InvocationError)
    }
}

pub fn get_custom_resolvers(local_context: Arc<LocalContext>) -> CustomResolvers {
    CustomResolvers { local_context }
}

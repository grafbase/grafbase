use crate::bridge::Bridge;
use grafbase_runtime::{
    self,
    custom_resolvers::{
        CustomResolverError, CustomResolverRequest, CustomResolverResponse, CustomResolversEngineInner,
    },
    ExecutionContext,
};

pub struct CustomResolvers {
    bridge: Bridge,
}

impl CustomResolvers {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(bridge_port: u16) -> grafbase_runtime::custom_resolvers::CustomResolversEngine {
        grafbase_runtime::custom_resolvers::CustomResolversEngine::new(Box::new(Self {
            bridge: Bridge::new(bridge_port),
        }))
    }
}

#[async_trait::async_trait(?Send)]
impl CustomResolversEngineInner for CustomResolvers {
    async fn invoke(
        &self,
        ctx: &ExecutionContext,
        request: CustomResolverRequest,
    ) -> Result<CustomResolverResponse, CustomResolverError> {
        self.bridge.request("/invoke-resolver", request).await.map_err(|error| {
            log::error!(ctx.request_id, "Resolver invocation failed with: {}", error);
            CustomResolverError::ServerError
        })
    }
}

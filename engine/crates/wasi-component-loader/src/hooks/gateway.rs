use core::fmt;

use http::HeaderMap;

use crate::{
    names::{COMPONENT_GATEWAY_REQUEST, GATEWAY_HOOK_FUNCTION},
    ComponentLoader, ContextMap, GuestResult,
};

use super::ComponentInstance;

/// The gateway hook is called right after authentication.
///
/// An instance of a function to be called from the Gateway level for the request.
/// The instance is meant to be separate for every request. The instance shares a memory space
/// with the guest, and cannot be shared with multiple requests.
pub struct GatewayHookInstance(ComponentInstance);

impl std::ops::Deref for GatewayHookInstance {
    type Target = ComponentInstance;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for GatewayHookInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Debug for GatewayHookInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "GatewayHookInstance { ... }".fmt(f)
    }
}

impl GatewayHookInstance {
    /// Creates a new instance for the gateway hook.
    pub async fn new(loader: &ComponentLoader) -> crate::Result<Self> {
        ComponentInstance::new(loader, COMPONENT_GATEWAY_REQUEST)
            .await
            .map(Self)
    }

    /// Calls the hook with the given parameters.
    pub async fn call(&mut self, context: ContextMap, headers: HeaderMap) -> crate::Result<(ContextMap, HeaderMap)> {
        let Some(hook) = self.get_hook::<_, (GuestResult<()>,)>(GATEWAY_HOOK_FUNCTION) else {
            return Ok((context, headers));
        };

        // adds the data to the shared memory
        let context = self.store.data_mut().push_resource(context)?;
        let headers = self.store.data_mut().push_resource(headers)?;

        // we need to take the pointers now, because a resource is not Copy and we need
        // the pointers to get the data back from the shared memory.
        let headers_rep = headers.rep();
        let context_rep = context.rep();

        let result = hook.call_async(&mut self.store, (context, headers)).await;

        if result.is_err() {
            self.poisoned = true;
        } else {
            hook.post_return_async(&mut self.store).await?;
        }

        result?.0?;

        // take the data back from the shared memory
        let context = self.store.data_mut().take_resource(context_rep)?;
        let headers = self.store.data_mut().take_resource(headers_rep)?;

        Ok((context, headers))
    }
}

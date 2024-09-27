use http::HeaderMap;
use tracing::Instrument;

use crate::{
    names::{GATEWAY_HOOK_FUNCTION, GATEWAY_REQUEST_INTERFACE},
    ComponentLoader, ContextMap, GuestResult,
};

use super::{component_instance, ComponentInstance};

component_instance!(GatewayComponentInstance: GATEWAY_REQUEST_INTERFACE);

impl GatewayComponentInstance {
    /// Called just before parsing and executing a gateway operation.
    ///
    /// # Arguments
    ///
    /// * `context` - A map containing the key-value context store for the request.
    /// * `headers` - A map containing the request headers.
    ///
    /// # Returns
    ///
    /// Returns a result containing a tuple of the processed context and headers,
    /// or an error if the operation fails.
    pub async fn on_gateway_request(
        &mut self,
        context: ContextMap,
        headers: HeaderMap,
    ) -> crate::Result<(ContextMap, HeaderMap)> {
        let Some(hook) = self.get_hook::<_, (GuestResult<()>,)>(GATEWAY_HOOK_FUNCTION) else {
            return Ok((context, headers));
        };

        let span = tracing::info_span!(GATEWAY_HOOK_FUNCTION);

        // adds the data to the shared memory
        let context = self.store.data_mut().push_resource(context)?;
        let headers = self.store.data_mut().push_resource(headers)?;

        // we need to take the pointers now, because a resource is not Copy and we need
        // the pointers to get the data back from the shared memory.
        let headers_rep = headers.rep();
        let context_rep = context.rep();

        let result = hook
            .call_async(&mut self.store, (context, headers))
            .instrument(span)
            .await;

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

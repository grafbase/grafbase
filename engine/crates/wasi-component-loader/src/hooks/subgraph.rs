use url::Url;

use crate::{
    context::SharedContextMap,
    names::{COMPONENT_SUBGRAPH_REQUEST, ON_SUBGRAGH_REQUEST_HOOK_FUNCTION},
    ComponentLoader, GuestResult,
};

use super::ComponentInstance;

/// Subgraph related hooks
pub struct SubgraphHookInstance(ComponentInstance);

impl std::ops::Deref for SubgraphHookInstance {
    type Target = ComponentInstance;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SubgraphHookInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SubgraphHookInstance {
    /// Creates a new instance for the subgraph hooks.
    pub async fn new(loader: &ComponentLoader) -> crate::Result<Self> {
        ComponentInstance::new(loader, COMPONENT_SUBGRAPH_REQUEST)
            .await
            .map(Self)
    }

    /// Called just before sending a HTTP request to a subgraph
    pub async fn on_subgraph_request(
        &mut self,
        context: SharedContextMap,
        method: http::Method,
        url: &Url,
        headers: http::HeaderMap,
    ) -> crate::Result<http::HeaderMap> {
        let Some(hook) = self.get_hook::<_, (GuestResult<()>,)>(ON_SUBGRAGH_REQUEST_HOOK_FUNCTION) else {
            return Ok(headers);
        };

        let url = url.to_string();
        let method = method.to_string();
        // adds the data to the shared memory
        let context = self.store.data_mut().push_resource(context)?;
        let headers = self.store.data_mut().push_resource(headers)?;

        // we need to take the pointers now, because a resource is not Copy and we need
        // the pointers to get the data back from the shared memory.
        let headers_rep = headers.rep();
        let context_rep = context.rep();

        let result = hook.call_async(&mut self.store, (context, method, url, headers)).await;

        if result.is_err() {
            self.poisoned = true;
        } else {
            hook.post_return_async(&mut self.store).await?;
        }

        result?.0?;

        // take the data back from the shared memory
        self.store.data_mut().take_resource::<SharedContextMap>(context_rep)?;
        let headers = self.store.data_mut().take_resource(headers_rep)?;

        Ok(headers)
    }
}

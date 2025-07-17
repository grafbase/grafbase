use futures::future::BoxFuture;

use crate::{
    Error, SharedContext,
    extension::{ContractsExtensionInstance, api::wit},
};

#[allow(unused_variables)]
impl ContractsExtensionInstance for super::ExtensionInstanceSince0_19_0 {
    fn construct<'a>(
        &'a mut self,
        context: SharedContext,
        key: &'a str,
        directives: Vec<wit::Directive<'a>>,
        subgraphs: Vec<wit::GraphqlSubgraphParam<'a>>,
    ) -> BoxFuture<'a, Result<Result<wit::Contract, String>, Error>> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;

            let context = self.store.data_mut().push_resource(context)?;

            let result = self
                .inner
                .grafbase_sdk_contracts()
                .call_construct(&mut self.store, context, key, &directives, &subgraphs)
                .await?;

            self.poisoned = false;
            Ok(result)
        })
    }
}

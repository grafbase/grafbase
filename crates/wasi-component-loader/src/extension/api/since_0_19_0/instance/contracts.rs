use futures::future::BoxFuture;

use crate::{
    LegacyWasmContext,
    extension::{ContractsExtensionInstance, api::wit},
};

#[allow(unused_variables)]
impl ContractsExtensionInstance for super::ExtensionInstanceSince0_19_0 {
    fn construct<'a>(
        &'a mut self,
        key: &'a str,
        directives: &'a [wit::Directive<'a>],
        subgraphs: Vec<wit::GraphqlSubgraphParam<'a>>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<wit::Contract, String>>> {
        Box::pin(async move {
            let context = self.store.data_mut().resources.push(LegacyWasmContext::default())?;

            let result = self
                .inner
                .grafbase_sdk_contracts()
                .call_construct(&mut self.store, context, key, directives, &subgraphs)
                .await?;

            Ok(result)
        })
    }
}

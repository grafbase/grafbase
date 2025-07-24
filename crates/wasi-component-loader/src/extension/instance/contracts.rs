use futures::future::BoxFuture;

use crate::{WasmContext, extension::api::wit};

#[allow(unused_variables)]
pub(crate) trait ContractsExtensionInstance {
    fn construct<'a>(
        &'a mut self,
        context: &'a WasmContext,
        key: &'a str,
        directives: &'a [wit::Directive<'a>],
        subgraphs: Vec<wit::GraphqlSubgraphParam<'a>>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<wit::Contract, String>>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }
}

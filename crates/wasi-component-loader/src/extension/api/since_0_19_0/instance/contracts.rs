use futures::future::BoxFuture;

use crate::extension::{ContractsExtensionInstance, api::wit};

#[allow(unused_variables)]
impl ContractsExtensionInstance for super::ExtensionInstanceSince0_19_0 {
    fn construct<'a>(
        &'a mut self,
        context: crate::SharedContext,
        directives: Vec<wit::Directive<'a>>,
        subgraphs: Vec<wit::GraphqlSubgraphParam<'a>>,
    ) -> BoxFuture<'a, Result<Result<wit::Contract, String>, crate::Error>> {
        Box::pin(async { std::unreachable!("Not supported by this SDK") })
    }
}

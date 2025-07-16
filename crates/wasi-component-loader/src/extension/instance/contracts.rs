use futures::future::BoxFuture;

use crate::{Error, SharedContext, extension::api::wit};

#[allow(unused_variables)]
pub(crate) trait ContractsExtensionInstance {
    fn construct<'a>(
        &'a mut self,
        context: SharedContext,
        directives: Vec<wit::Directive<'a>>,
        subgraphs: Vec<wit::GraphqlSubgraphParam<'a>>,
    ) -> BoxFuture<'a, Result<Result<wit::Contract, String>, Error>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }
}

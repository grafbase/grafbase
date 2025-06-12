use std::future::Future;

use engine_schema::Subgraph;
use error::GraphqlError;
use extension_catalog::ExtensionId;

use super::{Anything, ArgumentsId, Data, Field};

pub trait SelectionSetResolverExtension: Send + Sync + 'static {
    fn prepare<'ctx, F: Field<'ctx>>(
        &'ctx self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'ctx>,
        field: F,
    ) -> impl Future<Output = Result<Vec<u8>, GraphqlError>> + Send;

    fn resolve<'ctx, 'resp, 'f>(
        &'ctx self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: impl Iterator<Item = (ArgumentsId, impl Anything<'resp>)> + Send,
    ) -> impl Future<Output = Result<Data, GraphqlError>> + Send + 'f
    where
        'ctx: 'f;
}

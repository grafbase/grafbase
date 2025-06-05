use engine_error::GraphqlError;
use futures::future::BoxFuture;
use runtime::extension::Data;

use crate::{
    Error,
    extension::{
        SelectionSetResolverExtensionInstance,
        api::wit::{ArgumentsId, Field, FieldId},
    },
};

impl SelectionSetResolverExtensionInstance for super::ExtensionInstanceSince0_10_0 {
    fn selection_set_resolver_prepare(
        &mut self,
        _subgraph_name: &str,
        _field_id: FieldId,
        _fields: &[Field<'_>],
    ) -> BoxFuture<'_, Result<Result<Vec<u8>, GraphqlError>, Error>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    fn resolve_query_or_mutation_field(
        &mut self,
        _headers: http::HeaderMap,
        _subgraph_name: &str,
        _prepared: &[u8],
        _arguments: &[(ArgumentsId, &[u8])],
    ) -> BoxFuture<'_, Result<Result<Data, GraphqlError>, Error>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }
}

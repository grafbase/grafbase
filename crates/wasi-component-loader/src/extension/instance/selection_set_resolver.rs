use engine_error::GraphqlError;
use futures::future::BoxFuture;
use runtime::extension::Data;

use crate::extension::api::wit::{ArgumentsId, Field, FieldId};

#[allow(unused_variables)]
pub(crate) trait SelectionSetResolverExtensionInstance {
    fn selection_set_resolver_prepare<'a>(
        &'a mut self,
        subgraph_name: &'a str,
        field_id: FieldId,
        fields: &'a [Field<'a>],
    ) -> BoxFuture<'a, wasmtime::Result<Result<Vec<u8>, GraphqlError>>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    fn resolve_query_or_mutation_field<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        prepared: &'a [u8],
        arguments: &'a [(ArgumentsId, &'a [u8])],
    ) -> BoxFuture<'a, wasmtime::Result<Result<Data, GraphqlError>>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }
}

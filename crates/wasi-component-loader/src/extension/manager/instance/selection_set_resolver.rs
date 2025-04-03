use engine_error::GraphqlError;
use futures::future::BoxFuture;
use runtime::extension::Data;

use crate::{
    Error,
    extension::api::wit::{ArgumentsId, Field, FieldId},
};

pub(crate) trait SelectionSetResolverExtensionInstance {
    fn prepare<'a>(
        &'a mut self,
        subgraph_name: &'a str,
        field_id: FieldId,
        fields: &'a [Field<'a>],
    ) -> BoxFuture<'a, Result<Result<Vec<u8>, GraphqlError>, Error>>;

    fn resolve_query_or_mutation_field<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        prepared: &'a [u8],
        arguments: &'a [(ArgumentsId, &'a [u8])],
    ) -> BoxFuture<'a, Result<Result<Data, GraphqlError>, Error>>;
}

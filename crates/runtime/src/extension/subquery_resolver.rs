use std::future::Future;

use engine_schema::FieldDefinition;
use error::GraphqlError;

use crate::hooks::Anything;

use super::Data;

pub struct VariableId(pub u16);

pub trait SelectionSet<'a>: Sized + Send + 'a {
    type Field: Field<'a, SelectionSet = Self>;
    fn requires_typename(&self) -> bool;
    fn fields_ordered_by_parent_entity(&self) -> impl Iterator<Item = Self::Field>;
}

pub trait Field<'a>: Sized + Send + 'a {
    type SelectionSet: SelectionSet<'a>;
    fn alias(&self) -> Option<&'a str>;
    fn definition(&self) -> FieldDefinition<'a>;
    fn arguments(&self) -> Option<VariableId>;
    fn selection_set(&self) -> Self::SelectionSet;
}

pub trait SubQueryResolverExtension<Context: Send + Sync + 'static>: Send + Sync + 'static {
    fn prepare<'ctx>(
        &'ctx self,
        field_definition: FieldDefinition<'ctx>,
        selection_set: impl SelectionSet<'ctx>,
    ) -> impl Future<Output = Result<Vec<u8>, GraphqlError>> + Send;

    fn resolve_query_or_mutation_field<'ctx, 'resp, 'f>(
        &'ctx self,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        variables: impl Iterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Data, GraphqlError>> + Send + 'f
    where
        'ctx: 'f;
}

use std::future::Future;

use engine_schema::{FieldDefinition, Subgraph};
use error::GraphqlError;
use extension_catalog::ExtensionId;

use crate::hooks::Anything;

use super::Data;

pub trait SelectionSet<'a>: Sized + Send + 'a {
    type Field: Field<'a, SelectionSet = Self>;
    fn requires_typename(&self) -> bool;
    fn fields_ordered_by_parent_entity(&self) -> impl Iterator<Item = Self::Field>;
    // For test purposes. Don't use it for production code, it's just slower.
    fn as_dyn(&self) -> Box<dyn DynSelectionSet<'a>>;
}

pub trait Field<'a>: Sized + Send + 'a {
    type SelectionSet: SelectionSet<'a>;
    fn alias(&self) -> Option<&'a str>;
    fn definition(&self) -> FieldDefinition<'a>;
    fn arguments(&self) -> Option<ArgumentsId>;
    fn selection_set(&self) -> Option<Self::SelectionSet>;
}

pub trait DynSelectionSet<'a>: Send + 'a {
    fn requires_typename(&self) -> bool;
    fn fields_ordered_by_parent_entity(&self) -> Vec<Box<dyn DynField<'a>>>;
}

pub trait DynField<'a>: Send + 'a {
    fn alias(&self) -> Option<&'a str>;
    fn definition(&self) -> FieldDefinition<'a>;
    fn arguments(&self) -> Option<ArgumentsId>;
    fn selection_set(&self) -> Option<Box<dyn DynSelectionSet<'a>>>;
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ArgumentsId(pub u16);

pub trait SelectionSetResolverExtension<Context: Send + Sync + 'static>: Send + Sync + 'static {
    fn prepare<'ctx>(
        &'ctx self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        selection_set: impl SelectionSet<'ctx>,
    ) -> impl Future<Output = Result<Vec<u8>, GraphqlError>> + Send;

    fn resolve_query_or_mutation_field<'ctx, 'resp, 'f>(
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

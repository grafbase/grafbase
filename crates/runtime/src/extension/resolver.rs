use std::future::Future;

use engine_schema::{ExtensionDirective, FieldDefinition};
use error::GraphqlError;
use futures_util::stream::BoxStream;

use crate::{extension::Response, hooks::Anything};

pub trait SelectionSet<'a>: Sized + Send + 'a {
    type Field: Field<'a, SelectionSet = Self>;
    fn requires_typename(&self) -> bool;
    fn fields_ordered_by_parent_entity(&self) -> impl Iterator<Item = Self::Field>;
}

pub trait Field<'a>: Sized + Send + 'a {
    type SelectionSet: SelectionSet<'a>;
    fn alias(&self) -> Option<&'a str>;
    fn definition(&self) -> FieldDefinition<'a>;
    fn arguments(&self) -> Option<ArgumentsId>;
    fn selection_set(&self) -> Option<Self::SelectionSet>;
    // For test purposes. Don't use it for production code, it's just slower.
    fn as_dyn(&self) -> Box<dyn DynField<'a>>;
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

impl From<ArgumentsId> for u16 {
    fn from(id: ArgumentsId) -> Self {
        id.0
    }
}

pub trait ResolverExtension<Context: Send + Sync + 'static>: Send + Sync + 'static {
    fn prepare<'ctx, F: Field<'ctx>>(
        &'ctx self,
        context: &Context,
        directive: ExtensionDirective<'ctx>,
        directive_arguments: impl Anything<'ctx>,
        field: F,
    ) -> impl Future<Output = Result<Vec<u8>, GraphqlError>> + Send;

    fn resolve<'ctx, 'resp, 'f>(
        &'ctx self,
        context: &Context,
        directive: ExtensionDirective<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: impl Iterator<Item = (ArgumentsId, impl Anything<'resp>)> + Send,
    ) -> impl Future<Output = Response> + Send + 'f
    where
        'ctx: 'f;

    fn resolve_subscription<'ctx, 'resp, 'f>(
        &'ctx self,
        context: &Context,
        directive: ExtensionDirective<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: impl Iterator<Item = (ArgumentsId, impl Anything<'resp>)> + Send,
    ) -> impl Future<Output = BoxStream<'f, Response>> + Send + 'f
    where
        'ctx: 'f;
}

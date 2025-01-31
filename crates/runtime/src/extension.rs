use engine_schema::Subgraph;
use extension_catalog::ExtensionId;

use crate::{
    error::PartialGraphqlError,
    hooks::{Anything, EdgeDefinition},
};

pub struct ExtensionDirective<'a, Args> {
    pub name: &'a str,
    pub static_arguments: Args,
}

pub enum Data {
    JsonBytes(Vec<u8>),
    CborBytes(Vec<u8>),
}

#[allow(async_fn_in_trait)]
pub trait ExtensionRuntime: Send + Sync + 'static {
    type SharedContext: Clone + Send + Sync + 'static;

    async fn resolve_field<'a>(
        &self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'a>,
        context: &Self::SharedContext,
        field: EdgeDefinition<'a>,
        directive: ExtensionDirective<'a, impl Anything<'a>>,
        inputs: impl IntoIterator<Item: Anything<'a>> + Send,
    ) -> Result<Vec<Result<Data, PartialGraphqlError>>, PartialGraphqlError>;
}

impl ExtensionRuntime for () {
    type SharedContext = ();

    async fn resolve_field<'a>(
        &self,
        _extension_id: ExtensionId,
        _subgraph: Subgraph<'a>,
        _context: &Self::SharedContext,
        _field: EdgeDefinition<'a>,
        _directive: ExtensionDirective<'a, impl Anything<'a>>,
        _inputs: impl IntoIterator<Item: Anything<'a>> + Send,
    ) -> Result<Vec<Result<Data, PartialGraphqlError>>, PartialGraphqlError> {
        Err(PartialGraphqlError::internal_hook_error())
    }
}

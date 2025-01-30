use engine_schema::SubgraphId;
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
        subgraph_id: SubgraphId,
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
        _subgraph_id: SubgraphId,
        _context: &Self::SharedContext,
        _field: EdgeDefinition<'a>,
        _directive: ExtensionDirective<'a, impl Anything<'a>>,
        _inputs: impl IntoIterator<Item: Anything<'a>> + Send,
    ) -> Result<Vec<Result<Data, PartialGraphqlError>>, PartialGraphqlError> {
        Err(PartialGraphqlError::internal_hook_error())
    }
}

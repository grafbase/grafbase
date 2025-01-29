use std::future::Future;

use crate::{
    error::PartialGraphqlError,
    hooks::{Anything, EdgeDefinition},
};

use super::ExtensionId;

pub struct ExtensionDirective<'a, Args> {
    pub name: &'a str,
    pub static_arguments: Args,
}

pub enum Data {
    JsonBytes(Vec<u8>),
    CborBytes(Vec<u8>),
}

pub trait ExtensionRuntime: Send + Sync + 'static {
    type SharedContext: Clone + Send + Sync + 'static;

    fn resolve_field<'a>(
        &self,
        id: ExtensionId,
        subgraph_directives: impl IntoIterator<Item = ExtensionDirective<'a, impl Anything<'a>>> + Send,
        context: &Self::SharedContext,
        field: EdgeDefinition<'a>,
        directive: ExtensionDirective<'a, impl Anything<'a>>,
        inputs: impl IntoIterator<Item: Anything<'a>> + Send,
    ) -> impl Future<Output = Result<Vec<Result<Data, PartialGraphqlError>>, PartialGraphqlError>> + Send;
}

impl ExtensionRuntime for () {
    type SharedContext = ();

    async fn resolve_field<'a>(
        &self,
        _id: ExtensionId,
        _subgraph_directives: impl IntoIterator<Item = ExtensionDirective<'a, impl Anything<'a>>> + Send,
        _context: &Self::SharedContext,
        _field: EdgeDefinition<'a>,
        _directive: ExtensionDirective<'a, impl Anything<'a>>,
        _inputs: impl IntoIterator<Item: Anything<'a>> + Send,
    ) -> Result<Vec<Result<Data, PartialGraphqlError>>, PartialGraphqlError> {
        Err(PartialGraphqlError::internal_hook_error())
    }
}

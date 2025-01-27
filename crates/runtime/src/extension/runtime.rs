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
    fn resolve_field<'a>(
        &self,
        id: ExtensionId,
        field: EdgeDefinition<'a>,
        directive: ExtensionDirective<'a, impl Anything<'a>>,
        inputs: impl IntoIterator<Item: Anything<'a>> + Send,
    ) -> Result<Vec<Result<Data, PartialGraphqlError>>, PartialGraphqlError>;
}

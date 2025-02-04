use std::{collections::HashMap, future::Future, sync::Arc};

use engine_schema::Subgraph;
use extension_catalog::ExtensionId;

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord, id_derives::Id)]
pub struct AuthorizerId(u16);

use crate::{
    error::{ErrorResponse, PartialGraphqlError},
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

    fn resolve_field<'a>(
        &self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'a>,
        context: &Self::SharedContext,
        field: EdgeDefinition<'a>,
        directive: ExtensionDirective<'a, impl Anything<'a>>,
        inputs: impl IntoIterator<Item: Anything<'a>> + Send,
    ) -> impl Future<Output = Result<Vec<Result<Data, PartialGraphqlError>>, PartialGraphqlError>> + Send;

    fn authenticate(
        &self,
        _extension_id: ExtensionId,
        _authorizer_id: AuthorizerId,
        _headers: Arc<http::HeaderMap>,
    ) -> impl Future<Output = Result<HashMap<String, serde_json::Value>, ErrorResponse>> + Send;
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
        Err(PartialGraphqlError::internal_extension_error())
    }

    async fn authenticate(
        &self,
        _extension_id: ExtensionId,
        _authorizer_id: AuthorizerId,
        _headers: Arc<http::HeaderMap>,
    ) -> Result<HashMap<String, serde_json::Value>, ErrorResponse> {
        Err(ErrorResponse {
            status: http::StatusCode::INTERNAL_SERVER_ERROR,
            errors: vec![PartialGraphqlError::internal_extension_error()],
        })
    }
}

use std::{collections::HashMap, future::Future};

use engine_schema::{FieldDefinition, Subgraph};
use extension_catalog::ExtensionId;

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord, id_derives::Id)]
pub struct AuthorizerId(u16);

use crate::{
    error::{ErrorResponse, PartialGraphqlError},
    hooks::Anything,
};

pub enum Data {
    JsonBytes(Vec<u8>),
    CborBytes(Vec<u8>),
}

pub struct ExtensionFieldDirective<'a, Args> {
    pub extension_id: ExtensionId,
    pub subgraph: Subgraph<'a>,
    pub field: FieldDefinition<'a>,
    pub name: &'a str,
    pub arguments: Args,
}

#[allow(async_fn_in_trait)]
pub trait ExtensionRuntime: Send + Sync + 'static {
    type SharedContext: Send + Sync + 'static;

    /// Resolve a field through an extension. Lifetime 'ctx will be available for as long as the
    /// future lives, but 'resp lifetime won't. It provides access to the response data that is
    /// shared, without lock, so it's only temporarily available.
    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        context: &'ctx Self::SharedContext,
        directive: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
        inputs: impl IntoIterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Vec<Result<Data, PartialGraphqlError>>, PartialGraphqlError>> + Send + 'f
    where
        'ctx: 'f;

    fn authenticate(
        &self,
        _extension_id: ExtensionId,
        _authorizer_id: AuthorizerId,
        _headers: http::HeaderMap,
    ) -> impl Future<Output = Result<(http::HeaderMap, HashMap<String, serde_json::Value>), ErrorResponse>> + Send;
}

impl ExtensionRuntime for () {
    type SharedContext = ();

    #[allow(clippy::manual_async_fn)]
    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        _context: &'ctx Self::SharedContext,
        _directive_context: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
        _inputs: impl IntoIterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Vec<Result<Data, PartialGraphqlError>>, PartialGraphqlError>> + Send + 'f
    where
        'ctx: 'f,
    {
        async { Err(PartialGraphqlError::internal_extension_error()) }
    }

    async fn authenticate(
        &self,
        _extension_id: ExtensionId,
        _authorizer_id: AuthorizerId,
        _headers: http::HeaderMap,
    ) -> Result<(http::HeaderMap, HashMap<String, serde_json::Value>), ErrorResponse> {
        Err(ErrorResponse {
            status: http::StatusCode::INTERNAL_SERVER_ERROR,
            errors: vec![PartialGraphqlError::internal_extension_error()],
        })
    }
}

use std::{future::Future, sync::Arc};

use engine_schema::{DirectiveSite, FieldDefinition, Subgraph};
use extension_catalog::ExtensionId;
use futures_util::stream::BoxStream;

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord, id_derives::Id)]
pub struct AuthorizerId(u16);

use crate::hooks::Anything;
use error::{ErrorResponse, GraphqlError};

#[derive(Clone, Debug)]
pub enum Data {
    JsonBytes(Vec<u8>),
    CborBytes(Vec<u8>),
}

impl Data {
    pub fn as_cbor(&self) -> Option<&[u8]> {
        match self {
            Data::JsonBytes(_) => None,
            Data::CborBytes(bytes) => Some(bytes),
        }
    }
}

pub struct ExtensionFieldDirective<'a, Args> {
    pub extension_id: ExtensionId,
    pub subgraph: Subgraph<'a>,
    pub field: FieldDefinition<'a>,
    pub name: &'a str,
    pub arguments: Args,
}

pub struct QueryElement<'a, A> {
    pub site: DirectiveSite<'a>,
    pub arguments: A,
}

#[derive(Debug)]
pub enum AuthorizationDecisions {
    GrantAll,
    DenyAll(GraphqlError),
    DenySome {
        element_to_error: Vec<(u32, u32)>,
        errors: Vec<GraphqlError>,
    },
}

#[derive(Clone, Debug)]
pub enum Token {
    Anonymous,
    Bytes(Vec<u8>),
}

impl Token {
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Token::Anonymous => None,
            Token::Bytes(bytes) => Some(bytes),
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait ExtensionRuntime<Ctx>: Send + Sync + 'static {
    type SharedContext: Send + Sync + 'static;

    /// Resolve a field through an extension. Lifetime 'ctx will be available for as long as the
    /// future lives, but 'resp lifetime won't. It provides access to the response data that is
    /// shared, without lock, so it's only temporarily available.
    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        headers: http::HeaderMap,
        directive: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
        inputs: impl Iterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Vec<Result<Data, GraphqlError>>, GraphqlError>> + Send + 'f
    where
        'ctx: 'f;

    fn resolve_subscription<'ctx, 'f>(
        &'ctx self,
        headers: http::HeaderMap,
        directive: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
    ) -> impl Future<Output = Result<BoxStream<'f, Result<Arc<Data>, GraphqlError>>, GraphqlError>> + Send + 'f
    where
        'ctx: 'f;

    fn authenticate(
        &self,
        _extension_id: ExtensionId,
        _authorizer_id: AuthorizerId,
        _headers: http::HeaderMap,
    ) -> impl Future<Output = Result<(http::HeaderMap, Token), ErrorResponse>> + Send;

    fn authorize_query<'ctx, 'fut, Groups, QueryElements, Arguments>(
        &'ctx self,
        extension_id: ExtensionId,
        ctx: &'ctx Ctx,
        wasm_context: &'ctx Self::SharedContext,
        elements_grouped_by_directive_name: Groups,
    ) -> impl Future<Output = Result<AuthorizationDecisions, ErrorResponse>> + Send + 'fut
    where
        'ctx: 'fut,
        Groups: ExactSizeIterator<Item = (&'ctx str, QueryElements)>,
        QueryElements: ExactSizeIterator<Item = QueryElement<'ctx, Arguments>>,
        Arguments: Anything<'ctx>;

    fn authorize_response<'ctx, 'fut>(
        &'ctx self,
        extension_id: ExtensionId,
        ctx: &'ctx Ctx,
        wasm_context: &'ctx Self::SharedContext,
        directive_name: &'ctx str,
        directive_site: DirectiveSite<'ctx>,
        items: impl IntoIterator<Item: Anything<'ctx>>,
    ) -> impl Future<Output = Result<AuthorizationDecisions, GraphqlError>> + Send + 'fut
    where
        'ctx: 'fut;
}

#[allow(refining_impl_trait)]
impl<Ctx: Send + Sync + 'static> ExtensionRuntime<Ctx> for () {
    type SharedContext = ();

    #[allow(clippy::manual_async_fn)]
    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        _headers: http::HeaderMap,
        _directive_context: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
        _inputs: impl Iterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Vec<Result<Data, GraphqlError>>, GraphqlError>> + Send + 'f
    where
        'ctx: 'f,
    {
        async { Err(GraphqlError::internal_extension_error()) }
    }

    async fn authenticate(
        &self,
        _extension_id: ExtensionId,
        _authorizer_id: AuthorizerId,
        _headers: http::HeaderMap,
    ) -> Result<(http::HeaderMap, Token), ErrorResponse> {
        Err(ErrorResponse {
            status: http::StatusCode::INTERNAL_SERVER_ERROR,
            errors: vec![GraphqlError::internal_extension_error()],
        })
    }

    async fn resolve_subscription<'ctx, 'f>(
        &'ctx self,
        _: http::HeaderMap,
        _: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
    ) -> Result<BoxStream<'f, Result<Arc<Data>, GraphqlError>>, GraphqlError>
    where
        'ctx: 'f,
    {
        Err(GraphqlError::internal_extension_error())
    }

    #[allow(clippy::manual_async_fn)]
    fn authorize_query<'ctx, 'fut, Groups, QueryElements, Arguments>(
        &'ctx self,
        _: ExtensionId,
        _: &'ctx Ctx,
        _: &'ctx Self::SharedContext,
        _: Groups,
    ) -> impl Future<Output = Result<AuthorizationDecisions, ErrorResponse>> + Send + 'fut
    where
        'ctx: 'fut,
        Groups: ExactSizeIterator<Item = (&'ctx str, QueryElements)>,
        QueryElements: ExactSizeIterator<Item = QueryElement<'ctx, Arguments>>,
        Arguments: Anything<'ctx>,
    {
        async {
            Err(ErrorResponse {
                status: http::StatusCode::INTERNAL_SERVER_ERROR,
                errors: vec![GraphqlError::internal_extension_error()],
            })
        }
    }

    #[allow(clippy::manual_async_fn)]
    fn authorize_response<'ctx, 'fut>(
        &'ctx self,
        _: ExtensionId,
        _: &'ctx Ctx,
        _: &'ctx Self::SharedContext,
        _: &'ctx str,
        _: DirectiveSite<'_>,
        _: impl IntoIterator<Item: Anything<'ctx>>,
    ) -> impl Future<Output = Result<AuthorizationDecisions, GraphqlError>> + Send + 'fut
    where
        'ctx: 'fut,
    {
        async { Err(GraphqlError::internal_extension_error()) }
    }
}

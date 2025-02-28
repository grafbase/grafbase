use engine_schema::{Definition, FieldDefinition, Subgraph};
use extension_catalog::ExtensionId;
use futures_util::stream::BoxStream;

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord, id_derives::Id)]
pub struct AuthorizerId(u16);

use crate::{
    error::{ErrorResponse, PartialGraphqlError},
    hooks::Anything,
};

#[derive(Debug)]
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

pub struct DirectiveSite<'a, A> {
    pub definition: Definition<'a>,
    pub arguments: A,
}

pub enum AuthorizationDecisions {
    GrantAll,
    DenyAll(PartialGraphqlError),
    DenySome {
        element_to_error: Vec<(u32, u32)>,
        errors: Vec<PartialGraphqlError>,
    },
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
        inputs: impl Iterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Vec<Result<Data, PartialGraphqlError>>, PartialGraphqlError>> + Send + 'f
    where
        'ctx: 'f;

    fn resolve_subscription<'ctx, 'f>(
        &'ctx self,
        context: &'ctx Self::SharedContext,
        directive: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
    ) -> impl Future<Output = Result<BoxStream<'f, Result<Data, PartialGraphqlError>>, PartialGraphqlError>> + Send + 'f
    where
        'ctx: 'f;

    fn authenticate(
        &self,
        _extension_id: ExtensionId,
        _authorizer_id: AuthorizerId,
        _headers: http::HeaderMap,
    ) -> impl Future<Output = Result<(http::HeaderMap, Vec<u8>), ErrorResponse>> + Send;

    fn authorize_query<'ctx>(
        &'ctx self,
        context: &'ctx Self::SharedContext,
        extension_id: ExtensionId,
        // (directive name, (definition, arguments))
        elements: impl IntoIterator<
            Item = (
                &'ctx str,
                impl IntoIterator<Item = DirectiveSite<'ctx, impl Anything<'ctx>>> + Send + 'ctx,
            ),
        > + Send
        + 'ctx,
    ) -> impl Future<Output = Result<AuthorizationDecisions, ErrorResponse>> + Send;
}

#[allow(refining_impl_trait)]
impl ExtensionRuntime for () {
    type SharedContext = ();

    #[allow(clippy::manual_async_fn)]
    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        _context: &'ctx Self::SharedContext,
        _directive_context: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
        _inputs: impl Iterator<Item: Anything<'resp>> + Send,
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
    ) -> Result<(http::HeaderMap, Vec<u8>), ErrorResponse> {
        Err(ErrorResponse {
            status: http::StatusCode::INTERNAL_SERVER_ERROR,
            errors: vec![PartialGraphqlError::internal_extension_error()],
        })
    }

    async fn resolve_subscription<'ctx, 'f>(
        &'ctx self,
        _: &'ctx Self::SharedContext,
        _: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
    ) -> Result<BoxStream<'f, Result<Data, PartialGraphqlError>>, PartialGraphqlError>
    where
        'ctx: 'f,
    {
        Err(PartialGraphqlError::internal_extension_error())
    }

    async fn authorize_query<'ctx>(
        &'ctx self,
        _context: &'ctx Self::SharedContext,
        _extension_id: ExtensionId,
        _elements: impl IntoIterator<
            Item = (
                &'ctx str,
                impl IntoIterator<Item = DirectiveSite<'ctx, impl Anything<'ctx>>> + Send + 'ctx,
            ),
        > + Send
        + 'ctx,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        Err(ErrorResponse {
            status: http::StatusCode::INTERNAL_SERVER_ERROR,
            errors: vec![PartialGraphqlError::internal_extension_error()],
        })
    }
}

use std::{future::Future, sync::Arc};

use engine_schema::{DirectiveSite, FieldDefinition, Subgraph};
use extension_catalog::ExtensionId;
use futures_util::stream::BoxStream;

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord, id_derives::Id)]
pub struct AuthorizerId(u16);

use crate::{auth::LegacyToken, hooks::Anything};
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

/// It's not possible to provide a reference to wasmtime, it must be static and there are too many
/// layers to have good control over what's happening to use a transmute to get a &'static.
/// So this struct represents a lease that the engine grants on some value T that we expect to have
/// back. Depending on circumstances it may be one of the three possibilities.
pub enum Lease<T> {
    Owned(T),
    Shared(Arc<T>),
    SharedMut(Arc<tokio::sync::RwLock<T>>),
}

impl<T> From<T> for Lease<T> {
    fn from(t: T) -> Self {
        Lease::Owned(t)
    }
}

impl<T> Lease<T> {
    pub fn into_inner(self) -> Option<T> {
        match self {
            Lease::Owned(t) => Some(t),
            Lease::Shared(t) => Arc::into_inner(t),
            Lease::SharedMut(t) => Arc::into_inner(t).map(|t| t.into_inner()),
        }
    }

    pub async fn with_ref<R>(&self, f: impl AsyncFnOnce(&T) -> R) -> R
    where
        T: Send + Sync + 'static,
    {
        let mut _guard = None;
        let v = match self {
            Lease::Shared(v) => v,
            Lease::SharedMut(v) => {
                _guard = Some(v.read().await);
                _guard.as_deref().unwrap()
            }
            Lease::Owned(v) => v,
        };
        f(v).await
    }

    pub async fn with_ref_mut<R>(&mut self, f: impl AsyncFnOnce(Option<&mut T>) -> R) -> R
    where
        T: Send + Sync + 'static,
    {
        let mut _guard = None;
        let v = match self {
            Lease::Shared(_) => None,
            Lease::SharedMut(v) => {
                _guard = Some(v.write().await);
                _guard.as_deref_mut()
            }
            Lease::Owned(v) => Some(v),
        };
        f(v).await
    }
}

#[allow(async_fn_in_trait)]
pub trait ExtensionRuntime: Send + Sync + 'static {
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
        extension_id: ExtensionId,
        authorizer_id: AuthorizerId,
        headers: Lease<http::HeaderMap>,
    ) -> impl Future<Output = Result<(Lease<http::HeaderMap>, Token), ErrorResponse>> + Send;

    fn authorize_query<'ctx, 'fut, Groups, QueryElements, Arguments>(
        &'ctx self,
        extension_id: ExtensionId,
        wasm_context: &'ctx Self::SharedContext,
        headers: Lease<http::HeaderMap>,
        token: Lease<LegacyToken>,
        elements_grouped_by_directive_name: Groups,
    ) -> impl Future<Output = Result<(Lease<http::HeaderMap>, Lease<LegacyToken>, AuthorizationDecisions), ErrorResponse>>
           + Send
           + 'fut
    where
        'ctx: 'fut,
        Groups: ExactSizeIterator<Item = (&'ctx str, QueryElements)>,
        QueryElements: ExactSizeIterator<Item = QueryElement<'ctx, Arguments>>,
        Arguments: Anything<'ctx>;

    fn authorize_response<'ctx, 'fut>(
        &'ctx self,
        extension_id: ExtensionId,
        wasm_context: &'ctx Self::SharedContext,
        directive_name: &'ctx str,
        directive_site: DirectiveSite<'ctx>,
        items: impl IntoIterator<Item: Anything<'ctx>>,
    ) -> impl Future<Output = Result<AuthorizationDecisions, GraphqlError>> + Send + 'fut
    where
        'ctx: 'fut;
}

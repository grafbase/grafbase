use std::{future::Future, ops::Range, sync::Arc};

use engine_schema::{DirectiveSite, FieldDefinition, Subgraph};
use extension_catalog::ExtensionId;
use futures_util::stream::BoxStream;

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

#[derive(Clone, Debug)]
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

    pub fn as_ref(&self) -> TokenRef<'_> {
        match self {
            Token::Anonymous => TokenRef::Anonymous,
            Token::Bytes(bytes) => TokenRef::Bytes(bytes),
        }
    }
}

#[derive(Clone, Copy)]
pub enum TokenRef<'a> {
    Anonymous,
    Bytes(&'a [u8]),
}

impl TokenRef<'_> {
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            TokenRef::Anonymous => None,
            TokenRef::Bytes(bytes) => Some(bytes),
        }
    }

    pub fn to_owned(&self) -> Token {
        match self {
            TokenRef::Anonymous => Token::Anonymous,
            TokenRef::Bytes(bytes) => Token::Bytes(bytes.to_vec()),
        }
    }
}

pub struct QueryAuthorizationDecisions {
    pub extension_id: ExtensionId,
    pub query_elements_range: Range<usize>,
    pub decisions: AuthorizationDecisions,
}

#[allow(async_fn_in_trait)]
pub trait ExtensionRuntime: Send + Sync + 'static {
    type SharedContext: Send + Sync + 'static;

    /// Resolve a field through an extension. Lifetime 'ctx will be available for as long as the
    /// future lives, but 'resp lifetime won't. It provides access to the response data that is
    /// shared, without lock, so it's only temporarily available.
    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        subgraph_headers: http::HeaderMap,
        directive: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
        inputs: impl Iterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Vec<Result<Data, GraphqlError>>, GraphqlError>> + Send + 'f
    where
        'ctx: 'f;

    fn resolve_subscription<'ctx, 'f>(
        &'ctx self,
        subgraph_headers: http::HeaderMap,
        directive: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
    ) -> impl Future<Output = Result<BoxStream<'f, Result<Arc<Data>, GraphqlError>>, GraphqlError>> + Send + 'f
    where
        'ctx: 'f;

    fn authenticate(
        &self,
        extension_ids: &[ExtensionId],
        gateway_headers: http::HeaderMap,
    ) -> impl Future<Output = (http::HeaderMap, Result<Token, ErrorResponse>)> + Send;

    fn authorize_query<'ctx, 'fut, Extensions, Arguments>(
        &'ctx self,
        wasm_context: &'ctx Self::SharedContext,
        subgraph_headers: http::HeaderMap,
        token: TokenRef<'ctx>,
        extensions: Extensions,
        // (directive name, range within query_elements)
        directives: impl ExactSizeIterator<Item = (&'ctx str, Range<usize>)>,
        query_elements: impl ExactSizeIterator<Item = QueryElement<'ctx, Arguments>>,
    ) -> impl Future<Output = Result<(http::HeaderMap, Vec<QueryAuthorizationDecisions>), ErrorResponse>> + Send + 'fut
    where
        'ctx: 'fut,
        // (extension id, range within directives, range within query_elements)
        Extensions: IntoIterator<
                Item = (ExtensionId, Range<usize>, Range<usize>),
                IntoIter: ExactSizeIterator<Item = (ExtensionId, Range<usize>, Range<usize>)>,
            > + Send
            + Clone
            + 'ctx,
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

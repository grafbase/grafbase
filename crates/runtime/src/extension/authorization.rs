use std::{future::Future, ops::Range};

use engine_schema::{DirectiveSite, Subgraph};
use error::{ErrorResponse, GraphqlError};
use extension_catalog::ExtensionId;

use crate::extension::{AuthenticatedContext, AuthorizedContext};

use super::{Anything, TokenRef};

pub trait AuthorizationExtension: Send + Sync + 'static {
    fn authorize_query<'ctx, 'fut, Context, Extensions, Arguments>(
        &'ctx self,
        ctx: Context,
        subgraph_headers: http::HeaderMap,
        token: TokenRef<'ctx>,
        extensions: Extensions,
        // (directive name, range within query_elements)
        directives: impl ExactSizeIterator<Item = (&'ctx str, Range<usize>)>,
        query_elements: impl ExactSizeIterator<Item = QueryElement<'ctx, Arguments>>,
    ) -> impl Future<Output = Result<AuthorizeQuery, ErrorResponse>> + Send + 'fut
    where
        'ctx: 'fut,
        Context: AuthenticatedContext,
        // (extension id, range within directives, range within query_elements)
        Extensions: IntoIterator<
                Item = (ExtensionId, Range<usize>, Range<usize>),
                IntoIter: ExactSizeIterator<Item = (ExtensionId, Range<usize>, Range<usize>)>,
            > + Send
            + Clone
            + 'ctx,
        Arguments: Anything<'ctx>;

    fn authorize_response<'ctx, 'fut, Context>(
        &'ctx self,
        ctx: Context,
        extension_id: ExtensionId,
        directive_name: &'ctx str,
        directive_site: DirectiveSite<'ctx>,
        items: impl IntoIterator<Item: Anything<'ctx>>,
    ) -> impl Future<Output = Result<AuthorizationDecisions, GraphqlError>> + Send + 'fut
    where
        'ctx: 'fut,
        Context: AuthorizedContext;
}

#[derive(Clone, Debug)]
pub struct QueryElement<'a, A> {
    pub site: DirectiveSite<'a>,
    pub arguments: A,
    pub subgraph: Option<Subgraph<'a>>,
}

pub struct AuthorizeQuery {
    pub headers: http::HeaderMap,
    pub decisions: Vec<QueryAuthorizationDecisions>,
    pub context: Vec<(ExtensionId, Vec<u8>)>,
    pub state: Vec<(ExtensionId, Vec<u8>)>,
}

pub struct QueryAuthorizationDecisions {
    pub extension_id: ExtensionId,
    pub query_elements_range: Range<usize>,
    pub decisions: AuthorizationDecisions,
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

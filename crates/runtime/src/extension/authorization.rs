use std::{future::Future, ops::Range};

use engine_schema::DirectiveSite;
use error::{ErrorResponse, GraphqlError};
use extension_catalog::ExtensionId;

use super::{Anything, TokenRef};

pub trait AuthorizationExtension<Context: Send + Sync + 'static>: Send + Sync + 'static {
    fn authorize_query<'ctx, 'fut, Extensions, Arguments>(
        &'ctx self,
        ctx: &'ctx Context,
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
        ctx: &'ctx Context,
        extension_id: ExtensionId,
        directive_name: &'ctx str,
        directive_site: DirectiveSite<'ctx>,
        items: impl IntoIterator<Item: Anything<'ctx>>,
    ) -> impl Future<Output = Result<AuthorizationDecisions, GraphqlError>> + Send + 'fut
    where
        'ctx: 'fut;
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

pub struct QueryAuthorizationDecisions {
    pub extension_id: ExtensionId,
    pub query_elements_range: Range<usize>,
    pub decisions: AuthorizationDecisions,
}

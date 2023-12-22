use runtime::search::GraphqlCursor;

use super::{cursor::AtlasCursor, input::first, JsonMap};
use crate::{
    registry::resolvers::{ResolvedPaginationInfo, ResolverContext},
    ContextField, Error,
};

#[derive(Debug, Clone, Copy)]
pub(super) struct PaginationContext<'a> {
    context: &'a ContextField<'a>,
    resolver_context: &'a ResolverContext<'a>,
    forward: bool,
}

impl<'a> PaginationContext<'a> {
    pub(super) fn new(context: &'a ContextField<'a>, resolver_context: &'a ResolverContext<'a>) -> Self {
        let forward = first(context).is_some();

        Self {
            context,
            resolver_context,
            forward,
        }
    }
}

pub(super) fn paginate(
    ctx: PaginationContext<'_>,
    order_by: Option<&[JsonMap]>,
    documents: &[serde_json::Value],
    documents_fetched: usize,
) -> Result<ResolvedPaginationInfo, Error> {
    let start_cursor = documents
        .first()
        .and_then(serde_json::Value::as_object)
        .and_then(|first| AtlasCursor::new(ctx.context, ctx.resolver_context, order_by, first).ok())
        .map(GraphqlCursor::try_from)
        .transpose()?;

    let end_cursor = documents
        .last()
        .and_then(serde_json::Value::as_object)
        .and_then(|last| AtlasCursor::new(ctx.context, ctx.resolver_context, order_by, last).ok())
        .map(GraphqlCursor::try_from)
        .transpose()?;

    let has_previous_page = !ctx.forward && documents.len() < documents_fetched;
    let has_next_page = ctx.forward && documents.len() < documents_fetched;

    Ok(ResolvedPaginationInfo {
        start_cursor,
        end_cursor,
        has_next_page,
        has_previous_page,
    })
}

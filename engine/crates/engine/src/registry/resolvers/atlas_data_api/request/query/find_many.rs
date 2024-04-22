use graphql_cursor::GraphqlCursor;
use indexmap::IndexMap;
use serde::Serialize;
use serde_json::Value;

use super::AtlasQuery;
use crate::{
    names::OUTPUT_EDGE_CURSOR,
    registry::{
        resolvers::{
            atlas_data_api::{
                cursor::AtlasCursor,
                input,
                pagination::{self, PaginationContext},
                projection, JsonMap,
            },
            ResolvedValue, ResolverContext,
        },
        type_kinds::SelectionSetTarget,
    },
    ContextField, Error,
};

#[derive(Debug, Clone)]
struct Metadata {
    order_by: Option<Vec<JsonMap>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindMany {
    filter: JsonMap,
    projection: JsonMap,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<IndexMap<String, Value>>,
    limit: Option<usize>,
    #[serde(skip)]
    metadata: Metadata,
}

impl FindMany {
    pub fn new(ctx: &ContextField<'_>, resolver_ctx: &ResolverContext<'_>) -> Result<Self, Error> {
        match (input::first(ctx), input::last(ctx)) {
            (Some(_), Some(_)) => {
                return Err(Error::new("first and last parameters can't be both defined"));
            }
            (None, None) => {
                return Err(Error::new(
                    "please limit your selection by setting either the first or last parameter",
                ));
            }
            _ => (),
        }

        let selection_target: SelectionSetTarget<'_> = resolver_ctx.ty.try_into().unwrap();

        let selection_type = selection_target.field("edges").map(|field| field.ty().named_type());

        let selection_field = selection_type.as_ref().and_then(|output| output.field("node"));
        let selection_type = selection_field.map(|field| field.ty().named_type()).unwrap();

        let selection = ctx.look_ahead().field("edges").field("node").selection_fields();
        let projection = projection::project(ctx, selection.into_iter(), selection_type)?;
        let filter = input::filter(ctx)?;

        let order_by = input::order_by(ctx);
        let sort = input::sort(ctx, order_by.as_deref())?;

        let limit = input::first(ctx).or_else(|| input::last(ctx)).map(|limit| limit + 1);

        let metadata = Metadata { order_by };

        Ok(Self {
            filter,
            projection,
            sort,
            limit,
            metadata,
        })
    }

    pub fn convert_result(
        &self,
        ctx: &ContextField<'_>,
        resolver_ctx: &ResolverContext<'_>,
        result: &mut serde_json::Value,
    ) -> Result<ResolvedValue, Error> {
        let mut documents = result
            .as_object_mut()
            .and_then(|object| object.remove("documents"))
            .unwrap_or(serde_json::Value::Null);

        let pagination = if let serde_json::Value::Array(ref mut documents) = documents {
            let first = input::first(ctx);
            let last = input::last(ctx);
            let documents_fetched = documents.len();

            if let Some(limit) = first.or(last) {
                if documents_fetched == limit + 1 {
                    documents.pop();
                }
            }

            for document in documents.iter_mut().filter_map(serde_json::Value::as_object_mut) {
                let cursor = AtlasCursor::new(ctx, resolver_ctx, self.order_by(), document)?;

                let cursor = GraphqlCursor::try_from(cursor)
                    .ok()
                    .and_then(|cursor| serde_json::to_value(cursor).ok())
                    .unwrap_or_default();

                document.insert(OUTPUT_EDGE_CURSOR.to_string(), cursor);
            }

            if last.is_some() {
                documents.reverse();
            }

            Some(pagination::paginate(
                PaginationContext::new(ctx, resolver_ctx),
                self.order_by(),
                documents,
                documents_fetched,
            )?)
        } else {
            None
        };

        let mut resolved_value = ResolvedValue::new(documents);
        resolved_value.pagination = pagination;

        Ok(resolved_value)
    }

    fn order_by(&self) -> Option<&[JsonMap]> {
        self.metadata.order_by.as_deref()
    }
}

impl From<FindMany> for AtlasQuery {
    fn from(value: FindMany) -> Self {
        Self::FindMany(value)
    }
}

use std::{collections::HashSet, sync::Arc};

use dynamodb::{DynamoDBBatchersData, PaginatedCursor, PaginationOrdering};

use super::IdCursor;
use crate::{
    registry::{
        resolvers::{ResolvedPaginationDirection, ResolvedPaginationInfo, ResolvedValue},
        ModelName,
    },
    ContextExt, ContextField, Error,
};

pub(super) async fn by_ids(ctx: &ContextField<'_>, ids: &[String], ty: &ModelName) -> Result<ResolvedValue, Error> {
    let keys = ids.iter().map(|id| (id.clone(), id.clone())).collect::<Vec<_>>();
    let mut db_result = ctx
        .data::<Arc<DynamoDBBatchersData>>()?
        .loader
        .load_many(keys.clone())
        .await?;
    let type_name = ty.to_string();
    let result = keys
        .into_iter()
        .filter_map(|key| {
            db_result
                .remove(&key)
                // Resolvers on the model expect the type name...
                .map(|record| serde_json::json!({ &type_name: record }))
        })
        .collect::<Vec<_>>();

    Ok(ResolvedValue::new(serde_json::Value::Array(result)))
}

pub(super) async fn paginated_by_ids(
    ctx: &ContextField<'_>,
    ids: HashSet<String>,
    cursor: PaginatedCursor,
    ordering: PaginationOrdering,
    ty: &ModelName,
) -> Result<ResolvedValue, Error> {
    let mut ids = ids.into_iter().collect::<Vec<_>>();
    ids.sort();
    if ordering.is_desc() {
        ids.reverse();
    }
    let candidates: Vec<String> = match &cursor {
        PaginatedCursor::Forward { exclusive_last_key, .. } => {
            if let Some(after) = exclusive_last_key.as_ref() {
                ids.into_iter()
                    .filter(|id| if ordering.is_asc() { after < id } else { id < after })
                    .collect()
            } else {
                ids
            }
        }
        PaginatedCursor::Backward {
            exclusive_first_key, ..
        } => {
            let mut candidates = if let Some(before) = exclusive_first_key.as_ref() {
                ids.into_iter()
                    .filter(|id| if ordering.is_asc() { id < before } else { before < id })
                    .collect()
            } else {
                ids
            };
            // We retrieve items in the reversed order.
            //                         after
            //                           ┌───────► first (forward)
            //                           │
            //              ─────────────┼───────────────► Record order
            //                           │
            // last (backward) ◄─────────┘
            //                         before
            candidates.reverse();
            candidates
        }
    };

    let (items, has_more) = {
        let loader = &ctx.data::<Arc<DynamoDBBatchersData>>()?.loader;
        let mut items = Vec::new();
        let mut pos = 0;

        while items.len() <= cursor.limit() && pos < candidates.len() {
            // Adding one element to know whether there is more or not.
            let missing = cursor.limit() - items.len() + 1;
            let end = std::cmp::min(pos + missing, candidates.len());
            let keys = candidates[pos..end]
                .iter()
                .map(|id| (id.clone(), id.clone()))
                .collect::<Vec<_>>();
            pos = end;
            let mut result = loader.load_many(keys.clone()).await?;
            // Looks a bit silly, but it's the easiest way to keep the ordering of the candidates
            // we want for the specifying ordering & cursor.
            for key in keys {
                if let Some(item) = result.remove(&key) {
                    items.push((key, item));
                }
            }
        }

        let has_more = items.len() >= cursor.limit();
        items.truncate(cursor.limit());
        if cursor.is_backward() {
            // reverting back to the expected ordering
            items.reverse();
        }
        (items, has_more)
    };

    Ok({
        let pagination = ResolvedPaginationInfo::of(
            ResolvedPaginationDirection::from_paginated_cursor(&cursor),
            items.first().map(|((id, _), _)| IdCursor { id: id.to_string() }),
            items.last().map(|((id, _), _)| IdCursor { id: id.to_string() }),
            has_more,
        );
        let type_name = ty.to_string();
        let values = items
            .into_iter()
            // Resolvers on the model expect the type name...
            .map(|(_, item)| serde_json::json!({ &type_name: item }))
            .collect::<Vec<_>>();
        ResolvedValue::new(serde_json::Value::Array(values)).with_pagination(pagination)
    })
}

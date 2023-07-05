use std::sync::Arc;

use dynamodb::DynamoDBBatchersData;

use crate::{registry::resolvers::ResolvedValue, Context, Error};

pub(super) async fn by_ids(
    ctx: &Context<'_>,
    ids: &[String],
    type_name: &str,
) -> Result<ResolvedValue, Error> {
    let keys = ids
        .iter()
        .map(|id| (id.clone(), id.clone()))
        .collect::<Vec<_>>();
    let mut db_result = ctx
        .data::<Arc<DynamoDBBatchersData>>()?
        .loader
        .load_many(keys.clone())
        .await?;
    let result = keys
        .into_iter()
        .filter_map(|key| {
            db_result
                .remove(&key)
                // Resolvers on the model expect the type name...
                .map(|record| serde_json::json!({ type_name: record }))
        })
        .collect::<Vec<_>>();

    Ok(ResolvedValue::new(Arc::new(serde_json::Value::Array(
        result,
    ))))
}

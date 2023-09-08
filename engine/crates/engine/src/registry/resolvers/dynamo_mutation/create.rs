use std::sync::{atomic::AtomicUsize, Arc};

use engine_value::{ConstValue, Name};
use indexmap::IndexMap;

use crate::{
    registry::{
        resolvers::{ResolvedValue, ResolverContext},
        variables::VariableResolveDefinition,
        ModelName,
    },
    Context, Error,
};

#[derive(serde::Deserialize)]
struct CreateInput {
    input: IndexMap<Name, ConstValue>,
}

pub(super) async fn batch(
    ctx: &Context<'_>,
    resolver_ctx: &ResolverContext<'_>,
    last_resolver_value: Option<&ResolvedValue>,
    input: &VariableResolveDefinition,
    ty: &ModelName,
) -> Result<ResolvedValue, Error> {
    let meta_type = ctx.registry().lookup(ty)?;
    let input: Vec<CreateInput> = input.resolve(ctx, last_resolver_value)?;
    let increment = Arc::new(AtomicUsize::new(0));
    let (selections, transactions): (Vec<_>, Vec<_>) = input
        .into_iter()
        .map(|CreateInput { input }| {
            let super::RecursiveCreation { selection, transaction } = super::node_create(
                ctx,
                meta_type,
                *resolver_ctx.execution_id,
                increment.clone(),
                input,
                false,
            );
            (selection, transaction)
        })
        .unzip();
    // Not entirely sure what that does, but that's how DynamoMutationResolver::CreateNode works
    futures_util::future::try_join_all(selections).await?;
    let result = futures_util::future::try_join_all(transactions.into_iter().flatten()).await?;

    let ids = result
        .into_iter()
        .filter_map(|resolved| {
            let data = &resolved.data_resolved();
            if data
                .get("is_nested_relation")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_default()
            {
                return None;
            }
            data.get("id").and_then(|value| value.as_str().map(ToString::to_string))
        })
        .collect::<Vec<_>>();
    Ok(ResolvedValue::new(serde_json::json!({
        "ids": ids
    })))
}

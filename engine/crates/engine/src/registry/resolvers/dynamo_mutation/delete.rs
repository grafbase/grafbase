use std::{collections::HashSet, sync::Arc};

use dynamodb::{constant::INVERTED_INDEX_PK, DynamoDBBatchersData, PossibleChanges};
use graph_entities::NodeID;

use crate::{
    registry::{
        resolvers::ResolvedValue,
        variables::{id::ObfuscatedID, oneof::OneOf, VariableResolveDefinition},
        ModelName,
    },
    Context, ContextExt, ContextField, Error,
};

#[derive(serde::Deserialize)]
struct PostDeleteManyInput {
    by: OneOf<serde_json::Value>,
}

pub(super) async fn resolve_delete_nodes(
    ctx: &ContextField<'_>,
    last_resolver_value: Option<&ResolvedValue>,
    input: &VariableResolveDefinition,
    ty: &ModelName,
) -> Result<ResolvedValue, Error> {
    let Parsed { changes, deleted_ids } = generate_changes(ctx, input.resolve(ctx, last_resolver_value)?, ty).await?;

    let batchers = &ctx.data::<Arc<DynamoDBBatchersData>>()?;
    batchers.transaction_new.load_many(changes).await?;

    // FIXME: Should only return the ids/... of items that were actually deleted.
    Ok(ResolvedValue::new(serde_json::json!({
        "ids": deleted_ids
    })))
}

struct Parsed {
    changes: Vec<PossibleChanges>,
    deleted_ids: HashSet<String>,
}

async fn generate_changes(
    ctx: &ContextField<'_>,
    input: Vec<PostDeleteManyInput>,
    ty: &ModelName,
) -> Result<Parsed, Error> {
    let batchers = &ctx.data::<Arc<DynamoDBBatchersData>>()?;
    let meta_type = ctx.registry().lookup(ty)?;

    let mut deleted_ids = HashSet::new();
    let mut changes = Vec::new();
    for PostDeleteManyInput { by } in input {
        if by.name == "id" {
            let serde_json::Value::String(id_to_be_deleted) = by.value else {
                unreachable!()
            };

            let opaque_id = ObfuscatedID::expect(&id_to_be_deleted, &meta_type.name)
                .map_err(|err| err.into_server_error(ctx.item.pos))?;
            changes.push(PossibleChanges::delete_node(
                opaque_id.ty().to_string(),
                opaque_id.id().to_string(),
                None,
            ));
            deleted_ids.insert(id_to_be_deleted);
        } else {
            log::trace!(ctx.trace_id(), "constraint: {}", by.name);
            let constraint_id = meta_type
                .constraints
                .iter()
                .find(|constraint| constraint.name() == by.name)
                .and_then(|constraint| {
                    log::trace!(ctx.trace_id(), "constraint obj: {:#?}", constraint);
                    constraint.extract_id_from_by_input_field(
                        &meta_type.name,
                        &by.value.clone().try_into().expect("was a ConstValue before"),
                    )
                })
                .expect("constraint fields to be in the input");

            if let Some(mut item) = batchers
                .loader
                .load_one((constraint_id.to_string(), constraint_id.to_string()))
                .await?
            {
                let pk = item
                    .remove(INVERTED_INDEX_PK)
                    .and_then(|attr| attr.s)
                    .expect("must exist");
                let node_id = NodeID::from_owned(pk.clone()).unwrap();
                changes.push(PossibleChanges::delete_node(
                    node_id.ty().to_string(),
                    node_id.ulid().to_string(),
                    None,
                ));
                deleted_ids.insert(node_id.to_string());
            }
        }
    }
    Ok(Parsed { changes, deleted_ids })
}

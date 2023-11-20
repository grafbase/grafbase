use std::sync::{atomic::AtomicUsize, Arc};

use dynamodb::{constant::INVERTED_INDEX_PK, DynamoDBBatchersData};
use engine_value::{ConstValue, Name};
use graph_entities::{ConstraintID, NodeID};
use indexmap::IndexMap;
use itertools::Itertools;

use crate::{
    registry::{
        resolvers::{ResolvedValue, ResolverContext},
        variables::{id::ObfuscatedID, oneof::OneOf},
        ModelName, ObjectType,
    },
    Context, ContextExt, ContextField, Error, ServerError,
};

#[derive(serde::Deserialize)]
pub struct ParsedUpdateInput {
    input: IndexMap<Name, ConstValue>,
    by: OneOf<serde_json::Value>,
}

enum UpdateInput {
    ById(ById),
    ByConstraint(ByConstraint),
}

struct ById {
    id: String,
    input: IndexMap<Name, ConstValue>,
}

struct ByConstraint {
    constraint_id: ConstraintID<'static>,
    input: IndexMap<Name, ConstValue>,
}

impl ByConstraint {
    fn key(&self) -> (String, String) {
        (self.constraint_id.to_string(), self.constraint_id.to_string())
    }
}

struct Update {
    id: String,
    constraint_id: Option<ConstraintID<'static>>,
    input: IndexMap<Name, ConstValue>,
}

pub(super) async fn batch(
    ctx: &ContextField<'_>,
    resolver_ctx: &ResolverContext<'_>,
    input: Vec<ParsedUpdateInput>,
    ty: &ModelName,
) -> Result<ResolvedValue, Error> {
    let meta_type = ctx.registry().lookup(ty)?;
    let (by_ids, by_constraints) = partition_by_identifier(ctx, meta_type, input)?;

    let increment = Arc::new(AtomicUsize::new(0));
    let (ids, selections, transactions): (Vec<_>, Vec<_>, Vec<_>) = generate_updates(ctx, by_ids, by_constraints)
        .await?
        .into_iter()
        .map(
            |Update {
                 id,
                 constraint_id,
                 input,
             }| {
                super::node_update(
                    ctx,
                    meta_type,
                    *resolver_ctx.execution_id,
                    increment.clone(),
                    input,
                    id.clone(),
                    constraint_id,
                )
                .map(|super::RecursiveCreation { selection, transaction }| (id, selection, transaction))
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .multiunzip();

    // Not entirely sure what selections does, but that's how DynamoMutationResolver::UpdateNode works
    futures_util::future::try_join_all(selections).await?;
    futures_util::future::try_join_all(transactions.into_iter().flatten()).await?;

    Ok(ResolvedValue::new(serde_json::json!({
        "ids": ids
    })))
}

fn partition_by_identifier(
    ctx: &ContextField<'_>,
    meta_type: &ObjectType,
    input: Vec<ParsedUpdateInput>,
) -> Result<(Vec<ById>, Vec<ByConstraint>), ServerError> {
    input
        .into_iter()
        .map(|ParsedUpdateInput { input, by }| {
            if by.name == "id" {
                let serde_json::Value::String(id_to_be_deleted) = by.value else {
                    unreachable!()
                };
                ObfuscatedID::expect(&id_to_be_deleted, &meta_type.name)
                    .map_err(|err| err.into_server_error(ctx.item.pos))
                    .map(|opaque_id| {
                        UpdateInput::ById(ById {
                            id: opaque_id.to_string(),
                            input,
                        })
                    })
            } else {
                let constraint_id = meta_type
                    .constraints
                    .iter()
                    .find(|constraint| constraint.name() == by.name)
                    .and_then(|constraint| {
                        constraint.extract_id_from_by_input_field(
                            &meta_type.name,
                            &by.value.clone().try_into().expect("was a ConstValue before"),
                        )
                    })
                    .expect("constraint fields to be in the input");
                Ok(UpdateInput::ByConstraint(ByConstraint { constraint_id, input }))
            }
        })
        .collect::<Result<Vec<_>, ServerError>>()
        .map(|updates| {
            updates
                .into_iter()
                .map(|update| match update {
                    // Only used to partition the updates as Rust doesn't have Either in the stdl.
                    UpdateInput::ById(update) => Ok(update),
                    UpdateInput::ByConstraint(update) => Err(update),
                })
                .partition_result()
        })
}

async fn generate_updates(
    ctx: &ContextField<'_>,
    by_ids: Vec<ById>,
    by_constraints: Vec<ByConstraint>,
) -> Result<Vec<Update>, Error> {
    let mut updates: Vec<_> = by_ids
        .into_iter()
        .map(|ById { id, input }| Update {
            id,
            input,
            constraint_id: None,
        })
        .collect();
    let batchers = &ctx.data::<Arc<DynamoDBBatchersData>>()?;
    let mut key_to_items = batchers
        .loader
        .load_many(by_constraints.iter().map(ByConstraint::key).collect_vec())
        .await?;
    updates.extend(by_constraints.into_iter().filter_map(|by_constraint| {
        key_to_items.remove(&by_constraint.key()).map(|mut item| {
            let pk = item
                .remove(INVERTED_INDEX_PK)
                .and_then(|attr| attr.s)
                .expect("must exist");
            let node_id = NodeID::from_owned(pk).unwrap();
            Update {
                id: node_id.to_string(),
                constraint_id: Some(by_constraint.constraint_id),
                input: by_constraint.input,
            }
        })
    }));
    // If there are duplicates.
    if let Some(duplicates) = updates
        .iter()
        .into_group_map_by(|update| &update.id)
        .values()
        .find(|updates| updates.len() > 1)
    {
        Err(Error::new(format!(
            "Multiple updates target the same item: {}",
            duplicates.first().unwrap().id
        )))
    } else {
        Ok(updates)
    }
}

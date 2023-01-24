use crate::constant::{PK, RELATION_NAMES, SK};
use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::paginated::QueryValue;
use crate::runtime::Runtime;
use crate::utils::ConvertExtension;
use crate::{constant, CurrentDateTime, QueryKey};
use crate::{BatchGetItemLoaderError, TransactionError};
use crate::{DynamoDBBatchersData, DynamoDBContext};

use derivative::Derivative;
use dynomite::AttributeValue;
use futures::Future;
use futures_util::TryFutureExt;
use graph_entities::{normalize_constraint_value, ConstraintDefinition, ConstraintID, ConstraintType, NodeID};
use itertools::Itertools;
use log::info;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Add;
use std::pin::Pin;
use std::sync::{Arc, Weak};
use std::time::Duration;

cfg_if::cfg_if! {
    if #[cfg(not(feature = "local"))] {
        use crate::TxItem;

        mod dynamodb;
    } else {
        use crate::local::bridge_api;
        use crate::LocalContext;
    }
}

type TuplePartitionKeySortingKey = (String, String);

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct InsertNodeInput {
    id: String,
    ty: String,
    #[derivative(Debug = "ignore")]
    user_defined_item: HashMap<String, AttributeValue>,
    #[derivative(Debug = "ignore")]
    constraints: Vec<ConstraintDefinition>,
    current_datetime: CurrentDateTime,
}

impl PartialEq for InsertNodeInput {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id) && self.ty.eq(&other.ty)
    }
}

impl Hash for InsertNodeInput {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.ty.hash(state);
        state.finish();
    }
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct UpdateNodeInput {
    id: String,
    ty: String,
    #[derivative(Debug = "ignore")]
    user_defined_item: HashMap<String, AttributeValue>,
    #[derivative(Debug = "ignore")]
    increments: HashMap<String, AttributeValue>,
    by_id: Option<String>,
    current_datetime: CurrentDateTime,
}

impl PartialEq for UpdateNodeInput {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id) && self.ty.eq(&other.ty)
    }
}

impl Hash for UpdateNodeInput {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.ty.hash(state);
        state.finish();
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct DeleteNodeInput {
    id: String,
    ty: String,
    by_id: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum LinkNodeInput {
    Cache(LinkNodeCachedInput),
    NoCache(LinkNodeNoCacheInput),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LinkNodeNoCacheInput {
    from_id: String,
    from_ty: String,
    to_id: String,
    to_ty: String,
    relation_name: String,
    current_datetime: CurrentDateTime,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct LinkNodeCachedInput {
    from_id: String,
    from_ty: String,
    to_id: String,
    to_ty: String,
    relation_name: String,
    // May contain reserved fields, typically if the node existed already
    #[derivative(Debug = "ignore")]
    fields: HashMap<String, AttributeValue>,
    current_datetime: CurrentDateTime,
}

impl PartialEq for LinkNodeCachedInput {
    fn eq(&self, other: &Self) -> bool {
        self.from_id.eq(&other.from_id)
            && self.from_ty.eq(&other.from_ty)
            && self.to_id.eq(&other.to_id)
            && self.to_ty.eq(&other.to_ty)
            && self.relation_name.eq(&other.relation_name)
    }
}

impl Hash for LinkNodeCachedInput {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.from_id.hash(state);
        self.from_ty.hash(state);
        self.relation_name.hash(state);
        state.finish();
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct UnlinkNodeInput {
    from_id: String,
    from_ty: String,
    to_id: String,
    to_ty: String,
    relation_name: String,
    current_datetime: CurrentDateTime,
}

/// Public interface
#[derive(Debug, PartialEq, Clone, Hash)]
pub enum PossibleChanges {
    InsertNode(InsertNodeInput),
    UpdateNode(UpdateNodeInput),     // Unknow affected ids
    DeleteNode(DeleteNodeInput),     // Unknow affected ids
    LinkRelation(LinkNodeInput),     // One affected node
    UnlinkRelation(UnlinkNodeInput), // Unknown affected ids
}

impl Eq for PossibleChanges {}

impl PossibleChanges {
    pub const fn new_node(
        ty: String,
        id: String,
        user_defined_item: HashMap<String, AttributeValue>,
        constraints: Vec<ConstraintDefinition>,
        current_datetime: CurrentDateTime,
    ) -> Self {
        Self::InsertNode(InsertNodeInput {
            id,
            ty,
            user_defined_item,
            constraints,
            current_datetime,
        })
    }

    pub const fn update_node(
        ty: String,
        id: String,
        user_defined_item: HashMap<String, AttributeValue>,
        increments: HashMap<String, AttributeValue>,
        by_id: Option<String>,
        current_datetime: CurrentDateTime,
    ) -> Self {
        Self::UpdateNode(UpdateNodeInput {
            id,
            ty,
            user_defined_item,
            increments,
            by_id,
            current_datetime,
        })
    }

    pub const fn delete_node(ty: String, id: String, by_id: Option<String>) -> Self {
        Self::DeleteNode(DeleteNodeInput { id, ty, by_id })
    }

    pub const fn new_link_cached(
        from_ty: String,
        from_id: String,
        to_ty: String,
        to_id: String,
        relation_name: String,
        // May contain reserved fields, typically if the node existed already
        fields: HashMap<String, AttributeValue>,
        current_datetime: CurrentDateTime,
    ) -> Self {
        Self::LinkRelation(LinkNodeInput::Cache(LinkNodeCachedInput {
            from_id,
            from_ty,
            to_id,
            to_ty,
            relation_name,
            fields,
            current_datetime,
        }))
    }

    pub const fn unlink_node(
        from_ty: String,
        from_id: String,
        to_ty: String,
        to_id: String,
        relation_name: String,
        current_datetime: CurrentDateTime,
    ) -> Self {
        Self::UnlinkRelation(UnlinkNodeInput {
            from_id,
            from_ty,
            to_id,
            to_ty,
            relation_name,
            current_datetime,
        })
    }
}

type SelectionType<'a> = Pin<
    Box<dyn Future<Output = Result<HashMap<(String, String), InternalChanges>, BatchGetItemLoaderError>> + Send + 'a>,
>;

trait GetIds
where
    Self: Sized,
{
    /// Transform public interface to private one
    fn to_changes<'a>(self, batchers: &'a DynamoDBBatchersData, ctx: &'a DynamoDBContext) -> SelectionType<'a>;
}

impl GetIds for InsertNodeInput {
    /// For a InsertNode we'll need to:
    ///   - Insert the node
    ///   - Insert the constraints
    fn to_changes<'a>(self, _batchers: &'a DynamoDBBatchersData, _ctx: &'a DynamoDBContext) -> SelectionType<'a> {
        let pk = NodeID::new(&self.ty, &self.id).to_string();

        let mut result = HashMap::with_capacity(1 + self.constraints.len());

        for ConstraintDefinition {
            field,
            r#type: ConstraintType::Unique,
        } in self.constraints
        {
            if let Some(value) = self.user_defined_item.get(&field) {
                let contraint_id = ConstraintID::from_owned(self.ty.clone(), field.clone(), value.clone().into_json());
                result.insert(
                    (contraint_id.to_string(), contraint_id.to_string()),
                    InternalChanges::NodeConstraints(InternalNodeConstraintChanges::Insert(
                        InsertNodeConstraintInternalInput::Unique(InsertUniqueConstraint {
                            current_datetime: self.current_datetime.clone(),
                            target: pk.clone(),
                            user_defined_item: self.user_defined_item.clone(),
                            constraint_values: vec![value.clone().into_json().to_string()],
                            constraint_fields: vec![field],
                        }),
                    )),
                );
            }
        }

        result.insert(
            (pk.clone(), pk),
            InternalChanges::Node(InternalNodeChanges::Insert(InsertNodeInternalInput {
                current_datetime: self.current_datetime.clone(),
                id: self.id,
                ty: self.ty,
                user_defined_item: self.user_defined_item,
            })),
        );

        Box::pin(async { Ok(result) })
    }
}

impl GetIds for UpdateNodeInput {
    fn to_changes<'a>(self, batchers: &'a DynamoDBBatchersData, _ctx: &'a DynamoDBContext) -> SelectionType<'a> {
        let pk = NodeID::new(&self.ty, &self.id).to_string();

        let query_loader_reversed = &batchers.query_reversed;

        let select_entities_to_update = query_loader_reversed
            .load_one(QueryKey::new(pk.clone(), Vec::new()))
            .map_ok(|x| {
                x.into_iter()
                    .flat_map(|x| x.values.into_iter().map(|(_, val)| val))
                    .collect::<Vec<QueryValue>>()
            });

        Box::pin(async move {
            let ids = select_entities_to_update
                .await
                .map_err(|_| BatchGetItemLoaderError::UnknownError)?;

            let id_len = ids.len() + 1;
            let mut result = HashMap::with_capacity(id_len);

            // specific handling of the constraint that the ID was queried by (if any)
            // to prevent ACID issues
            if let Some(ref by_id) = self.by_id {
                if let Ok(constraint_id) = ConstraintID::try_from(by_id.clone()) {

                    let updated_value = self
                        .user_defined_item
                        .get(constraint_id.field())
                        .map(std::clone::Clone::clone)
                        .unwrap_or_default()
                        .into_json();

                    let new_constraint_id = constraint_id.clone().with_new_value(updated_value.clone());

                    if constraint_id == new_constraint_id {
                        result.insert(
                            (constraint_id.to_string(), constraint_id.to_string()),
                            InternalChanges::NodeConstraints(InternalNodeConstraintChanges::Update(
                                UpdateNodeConstraintInternalInput::Unique(UpdateUniqueConstraint {
                                    current_datetime: self.current_datetime.clone(),
                                    target: pk,
                                    user_defined_item: self.user_defined_item.clone(),
                                    increments: self.increments.clone(),
                                }),
                            )),
                        );
                    } else {
                        result.insert(
                            (by_id.clone(), by_id.clone()),
                            InternalChanges::NodeConstraints(InternalNodeConstraintChanges::Delete(
                                DeleteNodeConstraintInternalInput::Unit(DeleteUnitNodeConstraintInput {}),
                            )),
                        );

                        result.insert(
                            (new_constraint_id.to_string(), new_constraint_id.to_string()),
                            InternalChanges::NodeConstraints(InternalNodeConstraintChanges::Insert(
                                InsertNodeConstraintInternalInput::Unique(InsertUniqueConstraint {
                                    current_datetime: self.current_datetime.clone(),
                                    target: pk,
                                    user_defined_item: self.user_defined_item.clone(),
                                    constraint_values: vec![normalize_constraint_value(&updated_value)],
                                    constraint_fields: vec![new_constraint_id.field().to_string()],
                                }),
                            )),
                        );
                    }
                }
            }

            for val in ids {
                if let Some((pk, sk)) = val.node.and_then(|mut node| {
                    let pk = node.remove(PK).and_then(|x| x.s);
                    let sk = node.remove(SK).and_then(|x| x.s);
                    match (pk, sk) {
                        (Some(pk), Some(sk)) => Some((pk, sk)),
                        _ => None,
                    }
                }) {
                    let from = NodeID::from_owned(pk).map_err(|_| BatchGetItemLoaderError::UnknownError)?;

                    let from_ty = from.ty().to_string();
                    let from_id = from.ulid().to_string();

                    result.insert(
                        (from.to_string(), sk),
                        InternalChanges::Node(InternalNodeChanges::Update(UpdateNodeInternalInput {
                            current_datetime: self.current_datetime.clone(),
                            id: from_id.to_string(),
                            ty: from_ty.to_string(),
                            user_defined_item: self.user_defined_item.clone(),
                            increments: self.increments.clone(),
                        })),
                    );
                }

                for mut constraint in val.constraints {
                    let pk = constraint.remove(PK).and_then(|x| x.s);
                    let sk = constraint.remove(SK).and_then(|x| x.s);

                    if let (Some(pk), Some(sk)) = (pk, sk) {
                        // skip the constraint that we queried by if any, since it
                        // was already handled above
                        if let Some(ref by_id) = self.by_id {
                            if by_id == &pk {
                                continue;
                            }
                        }

                        if let Ok(constraint_id) = ConstraintID::try_from(pk.clone()) {
                            let updated_value = self
                                .user_defined_item
                                .get(constraint_id.field())
                                .map(std::clone::Clone::clone)
                                .unwrap_or_default()
                                .into_json();

                            let new_constraint_id = constraint_id.clone().with_new_value(updated_value.clone());

                            if constraint_id == new_constraint_id {
                                result.insert(
                                    (constraint_id.to_string(), constraint_id.to_string()),
                                    InternalChanges::NodeConstraints(InternalNodeConstraintChanges::Update(
                                        UpdateNodeConstraintInternalInput::Unique(UpdateUniqueConstraint {
                                            current_datetime: self.current_datetime.clone(),
                                            target: constraint
                                                .remove(constant::INVERTED_INDEX_PK)
                                                .and_then(|x| x.s)
                                                .unwrap(),
                                            user_defined_item: self.user_defined_item.clone(),
                                            increments: self.increments.clone(),
                                        }),
                                    )),
                                );
                            } else {
                                result.insert(
                                    (pk, sk),
                                    InternalChanges::NodeConstraints(InternalNodeConstraintChanges::Delete(
                                        DeleteNodeConstraintInternalInput::Unit(DeleteUnitNodeConstraintInput {}),
                                    )),
                                );

                                result.insert(
                                    (new_constraint_id.to_string(), new_constraint_id.to_string()),
                                    InternalChanges::NodeConstraints(InternalNodeConstraintChanges::Insert(
                                        InsertNodeConstraintInternalInput::Unique(InsertUniqueConstraint {
                                            current_datetime: self.current_datetime.clone(),
                                            target: constraint
                                                .remove(constant::INVERTED_INDEX_PK)
                                                .and_then(|x| x.s)
                                                .unwrap(),
                                            user_defined_item: self.user_defined_item.clone(),
                                            constraint_values: vec![normalize_constraint_value(&updated_value)],
                                            constraint_fields: vec![new_constraint_id.field().to_string()],
                                        }),
                                    )),
                                );
                            }
                        }
                    }
                }

                for mut relation in val.edges.into_iter().flat_map(|(_, x)| x.into_iter()) {
                    if let Some((pk, sk)) = {
                        let pk = relation.remove(PK).and_then(|x| x.s);
                        let sk = relation.remove(SK).and_then(|x| x.s);

                        match (pk, sk) {
                            (Some(pk), Some(sk)) => Some((pk, sk)),
                            _ => None,
                        }
                    } {
                        let from = NodeID::from_owned(pk).map_err(|_| BatchGetItemLoaderError::UnknownError)?;

                        let from_ty = from.ty().to_string();
                        let from_id = from.ulid().to_string();

                        result.insert(
                            (from.to_string(), sk),
                            InternalChanges::Node(InternalNodeChanges::Update(UpdateNodeInternalInput {
                                current_datetime: self.current_datetime.clone(),
                                id: from_id,
                                ty: from_ty,
                                user_defined_item: self.user_defined_item.clone(),
                                increments: self.increments.clone(),
                            })),
                        );
                    }
                }
            }

            Ok(result)
        })
    }
}

impl GetIds for DeleteNodeInput {
    fn to_changes<'a>(self, batchers: &'a DynamoDBBatchersData, _ctx: &'a DynamoDBContext) -> SelectionType<'a> {
        let id_to_be_deleted = NodeID::new(&self.ty, &self.id);
        let query_loader = &batchers.query;
        let query_loader_reversed = &batchers.query_reversed;

        let items_pk = query_loader.load_one(QueryKey::new(id_to_be_deleted.to_string(), Vec::new()));
        let items_sk = query_loader_reversed.load_one(QueryKey::new(id_to_be_deleted.to_string(), Vec::new()));

        let items_to_be_deleted = futures_util::future::try_join_all(vec![items_pk, items_sk]).map_ok(|x| {
            x.into_iter()
                .flatten()
                .flat_map(|x| x.values.into_iter().map(|(_, val)| val))
                .collect::<Vec<QueryValue>>()
        });

        // To remove a Node, we Remove the node and every relations (as the node is deleted)
        Box::pin(async move {
            let ids = items_to_be_deleted
                .await
                .map_err(|_| BatchGetItemLoaderError::UnknownError)?;

            let id_len = ids.len() + 1;
            let mut result = HashMap::with_capacity(id_len);

            // specific handling of the constraint that the ID was queried by (if any)
            // to prevent ACID issues
            if let Some(by_id) = self.by_id.clone() {
                result.insert(
                    (by_id.clone(), by_id),
                    InternalChanges::NodeConstraints(InternalNodeConstraintChanges::Delete(
                        DeleteNodeConstraintInternalInput::Unit(DeleteUnitNodeConstraintInput {}),
                    )),
                );
            }

            for val in ids {
                if let Some((pk, sk)) = val.node.and_then(|mut node| {
                    let pk = node.remove(PK).and_then(|x| x.s);
                    let sk = node.remove(SK).and_then(|x| x.s);

                    match (pk, sk) {
                        (Some(pk), Some(sk)) => Some((pk, sk)),
                        _ => None,
                    }
                }) {
                    let from = NodeID::from_owned(pk).map_err(|_| BatchGetItemLoaderError::UnknownError)?;

                    let from_ty = from.ty().to_string();
                    let from_id = from.ulid().to_string();

                    result.insert(
                        (from.to_string(), sk),
                        InternalChanges::Node(InternalNodeChanges::Delete(DeleteNodeInternalInput {
                            id: from_id.to_string(),
                            ty: from_ty.to_string(),
                        })),
                    );
                }

                for mut relation in val.edges.into_iter().flat_map(|(_, x)| x.into_iter()) {
                    if let Some((pk, sk)) = {
                        let pk = relation.remove(PK).and_then(|x| x.s);
                        let sk = relation.remove(SK).and_then(|x| x.s);

                        match (pk, sk) {
                            (Some(pk), Some(sk)) => Some((pk, sk)),
                            _ => None,
                        }
                    } {
                        let from = NodeID::from_owned(pk).map_err(|_| BatchGetItemLoaderError::UnknownError)?;
                        let to = NodeID::from_owned(sk).map_err(|_| BatchGetItemLoaderError::UnknownError)?;

                        let from_ty = from.ty().to_string();
                        let from_id = from.ulid().to_string();
                        let to_ty = to.ty().to_string();
                        let to_id = to.ulid().to_string();

                        result.insert(
                            (from.to_string(), to.to_string()),
                            InternalChanges::Relation(InternalRelationChanges::Delete(
                                DeleteRelationInternalInput::All(DeleteAllRelationsInternalInput {
                                    from_id,
                                    from_ty,
                                    to_id,
                                    to_ty,
                                    relation_names: None,
                                }),
                            )),
                        );
                    }
                }

                for mut constraint in val.constraints {
                    let pk = constraint.remove(PK).and_then(|x| x.s);
                    let sk = constraint.remove(SK).and_then(|x| x.s);

                    if let (Some(pk), Some(sk)) = (pk, sk) {
                        // skip the constraint that we queried by if any, since it
                        // was already handled above
                        if let Some(ref by_id) = self.by_id {
                            if by_id == &pk {
                                continue;
                            }
                        }

                        result.insert(
                            (pk, sk),
                            InternalChanges::NodeConstraints(InternalNodeConstraintChanges::Delete(
                                DeleteNodeConstraintInternalInput::Unit(DeleteUnitNodeConstraintInput {}),
                            )),
                        );
                    }
                }
            }

            Ok(result)
        })
    }
}

impl GetIds for LinkNodeCachedInput {
    fn to_changes<'a>(self, _batchers: &'a DynamoDBBatchersData, _ctx: &'a DynamoDBContext) -> SelectionType<'a> {
        let pk = NodeID::new(&self.from_ty, &self.from_id).to_string();
        let sk = NodeID::new(&self.to_ty, &self.to_id).to_string();

        Box::pin(async {
            Ok(HashMap::from([(
                (pk, sk),
                InternalChanges::Relation(InternalRelationChanges::Insert(InsertRelationInternalInput {
                    current_datetime: self.current_datetime,
                    from_id: self.from_id,
                    from_ty: self.from_ty,
                    to_id: self.to_id,
                    to_ty: self.to_ty,
                    relation_names: vec![self.relation_name],
                    fields: self.fields,
                })),
            )]))
        })
    }
}

impl GetIds for LinkNodeNoCacheInput {
    fn to_changes<'a>(self, batchers: &'a DynamoDBBatchersData, _ctx: &'a DynamoDBContext) -> SelectionType<'a> {
        let pk = NodeID::new(&self.from_ty, &self.from_id).to_string();
        let sk = NodeID::new(&self.to_ty, &self.to_id).to_string();

        Box::pin(async {
            let query_loader = &batchers.loader;
            let node = query_loader
                .load_one((pk.clone(), sk.clone()))
                .await
                .map_err(|_| BatchGetItemLoaderError::UnknownError)?
                .ok_or(BatchGetItemLoaderError::UnknownError)?;

            Ok(HashMap::from([(
                (pk, sk),
                InternalChanges::Relation(InternalRelationChanges::Insert(InsertRelationInternalInput {
                    current_datetime: self.current_datetime,
                    from_id: self.from_id,
                    from_ty: self.from_ty,
                    to_id: self.to_id,
                    to_ty: self.to_ty,
                    relation_names: vec![self.relation_name],
                    fields: node,
                })),
            )]))
        })
    }
}

impl GetIds for LinkNodeInput {
    fn to_changes<'a>(self, batchers: &'a DynamoDBBatchersData, ctx: &'a DynamoDBContext) -> SelectionType<'a> {
        match self {
            LinkNodeInput::Cache(a) => a.to_changes(batchers, ctx),
            LinkNodeInput::NoCache(a) => a.to_changes(batchers, ctx),
        }
    }
}

impl GetIds for UnlinkNodeInput {
    fn to_changes<'a>(self, batchers: &'a DynamoDBBatchersData, _ctx: &'a DynamoDBContext) -> SelectionType<'a> {
        let pk = NodeID::new(&self.from_ty, &self.from_id).to_string();
        let sk = NodeID::new(&self.to_ty, &self.to_id).to_string();

        Box::pin(async {
            let loader = &batchers.loader;
            let node = loader
                .load_one((pk.clone(), sk.clone()))
                .await?
                .and_then(|mut r| r.remove(constant::RELATION_NAMES))
                .and_then(|relations| relations.ss);

            match node {
                Some(relations) => {
                    // If it's the only relation remaining, we ask to delete everything
                    if relations.contains(&self.relation_name) && relations.len() == 1 {
                        Ok(HashMap::from([(
                            (pk, sk),
                            InternalChanges::Relation(InternalRelationChanges::Delete(
                                DeleteRelationInternalInput::All(DeleteAllRelationsInternalInput {
                                    from_id: self.from_id,
                                    from_ty: self.from_ty,
                                    to_id: self.to_id,
                                    to_ty: self.to_ty,
                                    relation_names: Some(relations),
                                }),
                            )),
                        )]))
                        // If it's not the only relation remaining, we ask to keep some
                    } else if relations.contains(&self.relation_name) {
                        Ok(HashMap::from([(
                            (pk, sk),
                            InternalChanges::Relation(InternalRelationChanges::Delete(
                                DeleteRelationInternalInput::Multiple(DeleteMultipleRelationsInternalInput {
                                    current_datetime: self.current_datetime,
                                    from_id: self.from_id,
                                    from_ty: self.from_ty,
                                    to_id: self.to_id,
                                    to_ty: self.to_ty,
                                    relation_names: vec![self.relation_name],
                                }),
                            )),
                        )]))
                    } else {
                        Ok(HashMap::new())
                    }
                }
                None => Ok(HashMap::new()),
            }
        })
    }
}
impl GetIds for PossibleChanges {
    fn to_changes<'a>(self, batchers: &'a DynamoDBBatchersData, ctx: &'a DynamoDBContext) -> SelectionType<'a> {
        match self {
            PossibleChanges::DeleteNode(a) => a.to_changes(batchers, ctx),
            PossibleChanges::InsertNode(a) => a.to_changes(batchers, ctx),
            PossibleChanges::UpdateNode(a) => a.to_changes(batchers, ctx),
            PossibleChanges::LinkRelation(a) => a.to_changes(batchers, ctx),
            PossibleChanges::UnlinkRelation(a) => a.to_changes(batchers, ctx),
        }
    }
}

#[cfg(not(feature = "local"))]
pub type TransactionOutput = HashMap<TxItem, AttributeValue>;

#[cfg(feature = "local")]
pub type TransactionOutput = (String, Vec<String>, Option<crate::local::types::OperationKind>);

pub type ToTransactionFuture<'a> =
    Pin<Box<dyn Future<Output = Result<TransactionOutput, ToTransactionError>> + Send + 'a>>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ToTransactionError {
    #[error("Internal error")]
    Unknown,
    #[error("Database internal error")]
    GetItemError(#[from] BatchGetItemLoaderError),
    #[error("{0}")]
    TransactionError(#[from] TransactionError),

    #[error(r#"The {} {} {} already taken on {} "{}""#,
        if .values.len() == 1 { "value" } else { "values"},
        .values.join(", "),
        if .values.len() == 1 { "is" } else { "are" },
        if .fields.len() == 1 { "field" } else { "fields" },
        .fields.join(", ")
    )]
    UniqueCondition {
        source: TransactionError,
        values: Vec<String>,
        fields: Vec<String>,
    },
}

pub trait ExecuteChangesOnDatabase
where
    Self: Sized,
{
    /// Multiple things are possible here
    /// DeleteNode -> Delete, affect: multiple
    /// InsertNode -> Put, affect: 1
    /// UpdateNode -> Update, affect: multiple
    /// UnlinkNode -> Update/Delete, affect: 1
    /// LinkNode -> Update, affect 1,
    fn to_transaction<'a>(
        self,
        batchers: &'a DynamoDBBatchersData,
        ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a>;
}

#[derive(Clone, Derivative, PartialEq)]
#[derivative(Debug)]
pub struct InsertNodeInternalInput {
    pub id: String,
    pub ty: String,
    #[derivative(Debug = "ignore")]
    pub user_defined_item: HashMap<String, AttributeValue>,
    pub current_datetime: CurrentDateTime,
}

#[derive(Derivative, PartialEq, Clone)]
#[derivative(Debug)]
pub struct UpdateNodeInternalInput {
    pub id: String,
    pub ty: String,
    #[derivative(Debug = "ignore")]
    pub user_defined_item: HashMap<String, AttributeValue>,
    #[derivative(Debug = "ignore")]
    pub increments: HashMap<String, AttributeValue>,
    pub current_datetime: CurrentDateTime,
}

impl UpdateNodeInternalInput {
    pub fn to_update_expression(
        current_datetime: CurrentDateTime,
        values: HashMap<String, AttributeValue>,
        increments: HashMap<String, AttributeValue>,
        exp_values: &mut HashMap<String, AttributeValue>,
        exp_names: &mut HashMap<String, String>,
    ) -> String {
        let values: HashMap<&String, &AttributeValue> = values
            .iter()
            // avoids cases where existing numerical fields
            // are added to updates
            .filter(|value| !increments.contains_key(value.0))
            .collect();

        let update_expression = values
            .into_iter()
            .filter(|(name, _)| !name.starts_with("__"))
            .chain(std::iter::once((
                &constant::UPDATED_AT.to_string(),
                &current_datetime.into_attr(),
            )))
            .unique_by(|(name, _)| (*name).to_string())
            .map(|(name, value)| {
                let idx = format!(":{}", name.as_str());
                let sanitized_name = format!("#{}", name.as_str());
                let result = format!("{sanitized_name}={idx}");
                exp_values.insert(idx, value.clone());
                exp_names.insert(sanitized_name, name.as_str().to_string());
                result
            })
            .join(",");

        let increment_expression = increments
            .into_iter()
            .map(|(name, value)| {
                let idx = format!(":{}", name.as_str());
                let sanitized_name = format!("#{}", name.as_str());
                let result = format!("{sanitized_name} {idx}");
                exp_values.insert(idx, value);
                exp_names.insert(sanitized_name, name.as_str().to_string());
                result
            })
            .join(",");

        let update_expression = if update_expression.is_empty() {
            String::new()
        } else {
            format!("set {update_expression}")
        };

        let increment_expression = if increment_expression.is_empty() {
            String::new()
        } else {
            format!("add {increment_expression}")
        };

        // since we insert all fields for updates at the moment
        // (including unchanged fields), both update_expression and
        // increment_expression cannot be empty at the same time
        format!("{update_expression} {increment_expression}")
    }
}

impl UpdateUniqueConstraint {
    pub fn to_update_expression(
        current_datetime: CurrentDateTime,
        values: HashMap<String, AttributeValue>,
        increments: HashMap<String, AttributeValue>,
        exp_values: &mut HashMap<String, AttributeValue>,
        exp_names: &mut HashMap<String, String>,
    ) -> String {
        let values: HashMap<&String, &AttributeValue> = values
            .iter()
            // avoids cases where existing numerical fields
            // are added to updates
            .filter(|value| !increments.contains_key(value.0))
            .collect();

        let update_expression = values
            .into_iter()
            .filter(|(name, _)| !name.starts_with("__"))
            .chain(std::iter::once((
                &constant::UPDATED_AT.to_string(),
                &current_datetime.into_attr(),
            )))
            .map(|(name, value)| {
                let idx = format!(":{}", name.as_str());
                let sanitized_name = format!("#{}", name.as_str());
                let result = format!("{sanitized_name}={idx}");
                exp_values.insert(idx, value.clone());
                exp_names.insert(sanitized_name, name.as_str().to_string());
                result
            })
            .join(",");

        let increment_expression = increments
            .into_iter()
            .map(|(name, value)| {
                let idx = format!(":{}", name.as_str());
                let sanitized_name = format!("#{}", name.as_str());
                let result = format!("{sanitized_name} {idx}");
                exp_values.insert(idx, value);
                exp_names.insert(sanitized_name, name.as_str().to_string());
                result
            })
            .join(",");

        let update_expression = if update_expression.is_empty() {
            String::new()
        } else {
            format!("set {update_expression}")
        };

        let increment_expression = if increment_expression.is_empty() {
            String::new()
        } else {
            format!("add {increment_expression}")
        };

        // since we insert all fields for updates at the moment
        // (including unchanged fields), both update_expression and
        // increment_expression cannot be empty at the same time
        format!("{update_expression} {increment_expression}")
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DeleteNodeInternalInput {
    pub id: String,
    pub ty: String,
}

#[derive(Derivative, PartialEq, Clone)]
#[derivative(Debug)]
pub struct InsertRelationInternalInput {
    pub from_id: String,
    pub from_ty: String,
    pub to_id: String,
    pub to_ty: String,
    pub relation_names: Vec<String>,
    #[derivative(Debug = "ignore")]
    pub fields: HashMap<String, AttributeValue>,
    pub current_datetime: CurrentDateTime,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DeleteRelationInternalInput {
    Multiple(DeleteMultipleRelationsInternalInput),
    All(DeleteAllRelationsInternalInput),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DeleteAllRelationsInternalInput {
    from_id: String,
    from_ty: String,
    to_id: String,
    to_ty: String,
    /// Used to specify which relation_names are deleted, used for Update addition
    /// If not there, the delete will still happen
    relation_names: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DeleteMultipleRelationsInternalInput {
    pub from_id: String,
    pub from_ty: String,
    pub to_id: String,
    pub to_ty: String,
    pub relation_names: Vec<String>,
    pub current_datetime: CurrentDateTime,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum UpdateRelation {
    #[allow(unused)]
    Add(String),
    Remove(String),
}
#[derive(Derivative, PartialEq, Clone)]
#[derivative(Debug)]
pub struct UpdateRelationInternalInput {
    pub from_id: String,
    pub from_ty: String,
    pub to_id: String,
    pub to_ty: String,
    pub relation_names: Vec<UpdateRelation>,
    #[derivative(Debug = "ignore")]
    pub user_defined_item: HashMap<String, AttributeValue>,
    pub current_datetime: CurrentDateTime,
}

impl UpdateRelationInternalInput {
    pub fn to_update_expression(
        current_datetime: CurrentDateTime,
        values: HashMap<String, AttributeValue>,
        exp_values: &mut HashMap<String, AttributeValue>,
        exp_names: &mut HashMap<String, String>,
        relation_names: Vec<UpdateRelation>,
        should_insert_private_fields: bool,
    ) -> String {
        let values_len = values.len();
        let update_expression = if values_len > 0 {
            let exp = values
                .into_iter()
                .filter(|(name, _)| should_insert_private_fields || !name.starts_with("__"))
                .chain(std::iter::once((
                    constant::UPDATED_AT.to_string(),
                    current_datetime.into_attr(),
                )))
                .unique_by(|(name, _)| name.to_string())
                .map(|(name, value)| {
                    let idx = format!(":{}", name.as_str());
                    let sanitized_name = format!("#{}", name.as_str());
                    let result = format!("{sanitized_name}={idx}");
                    exp_values.insert(idx, value);
                    exp_names.insert(sanitized_name, name.as_str().to_string());
                    result
                })
                .join(",");

            format!("set {exp}")
        } else {
            String::new()
        };

        let update_relation_expressions = if relation_names.is_empty() {
            String::new()
        } else {
            exp_names.insert("#relation_names".to_string(), RELATION_NAMES.to_string());
            let (removed, added): (Vec<String>, Vec<String>) =
                relation_names.into_iter().partition_map(|relation| match relation {
                    UpdateRelation::Add(a) => itertools::Either::Right(a),
                    UpdateRelation::Remove(a) => itertools::Either::Left(a),
                });

            let add_expression = added
                .into_iter()
                .map(|a| {
                    let idx = format!(":{}", &a);
                    let result = format!("ADD #relation_names {idx}");
                    exp_values.insert(
                        idx,
                        AttributeValue {
                            ss: Some(vec![a]),
                            ..Default::default()
                        },
                    );
                    result
                })
                .join(" ");

            if removed.is_empty() {
                add_expression
            } else {
                let idx = ":__relation_names_deleted".to_string();

                exp_values.insert(
                    idx,
                    AttributeValue {
                        ss: Some(removed),
                        ..Default::default()
                    },
                );

                format!("{add_expression} DELETE #relation_names :__relation_names_deleted")
            }
        };

        format!("{update_expression} {update_relation_expressions}")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum InternalNodeChanges {
    Insert(InsertNodeInternalInput),
    Update(UpdateNodeInternalInput), // Unknow affected ids
    Delete(DeleteNodeInternalInput), // Unknow affected ids
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// Delete every constraint of a Node
pub struct DeleteUnitNodeConstraintInput {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DeleteNodeConstraintInternalInput {
    Unit(DeleteUnitNodeConstraintInput),
}

#[derive(Debug, Clone, Derivative, PartialEq)]
pub struct InsertUniqueConstraint {
    /// The unique constraint targets one entity
    pub(crate) target: String,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    pub user_defined_item: HashMap<String, AttributeValue>,
    pub current_datetime: CurrentDateTime,
    pub constraint_fields: Vec<String>,
    pub constraint_values: Vec<String>,
}

#[derive(Debug, Clone, Derivative, PartialEq)]
pub struct UpdateUniqueConstraint {
    /// The unique constraint targets one entity
    pub(crate) target: String,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    pub user_defined_item: HashMap<String, AttributeValue>,
    #[derivative(Debug = "ignore", PartialEq = "ignore")]
    pub increments: HashMap<String, AttributeValue>,
    pub current_datetime: CurrentDateTime,
}

#[derive(Debug, PartialEq, Clone)]
pub enum InsertNodeConstraintInternalInput {
    Unique(InsertUniqueConstraint),
}

#[derive(Debug, PartialEq, Clone)]
pub enum UpdateNodeConstraintInternalInput {
    Unique(UpdateUniqueConstraint),
}

#[derive(Debug, PartialEq, Clone)]
pub enum InternalNodeConstraintChanges {
    Insert(InsertNodeConstraintInternalInput),
    Update(UpdateNodeConstraintInternalInput),
    Delete(DeleteNodeConstraintInternalInput), // Unknow affected ids
}

/// Private interface
#[derive(Debug, PartialEq, Clone)]
pub enum InternalChanges {
    Node(InternalNodeChanges),
    Relation(InternalRelationChanges),
    NodeConstraints(InternalNodeConstraintChanges),
}

#[derive(Debug, thiserror::Error)]
pub enum PossibleChangesInternalError {
    #[error("Internal error")]
    Unknown,
    #[error("You try to insert multiple node at the same time")]
    MultipleInsertWithSameNode,
    #[error("You try to insert and delete a node at the same time")]
    InsertAndDelete,
    #[error("You try to delete the same node multiple time")]
    MultipleDeleteWithSameNode,
    #[error("You can't compare node and relation")]
    NodeAndRelationCompare,
}

impl Add<InsertNodeInternalInput> for UpdateNodeInternalInput {
    type Output = InsertNodeInternalInput;

    fn add(self, rhs: InsertNodeInternalInput) -> Self::Output {
        Self::Output {
            current_datetime: self.current_datetime,
            id: self.id,
            ty: self.ty,
            user_defined_item: {
                let mut update_into_insert = rhs.user_defined_item;
                update_into_insert.extend(self.user_defined_item);
                update_into_insert
            },
        }
    }
}

impl Add<UpdateNodeInternalInput> for InsertNodeInternalInput {
    type Output = Self;
    fn add(self, rhs: UpdateNodeInternalInput) -> Self::Output {
        rhs + self
    }
}

impl Add<Self> for UpdateNodeInternalInput {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            current_datetime: self.current_datetime,
            id: self.id,
            ty: self.ty,
            user_defined_item: {
                let mut update_into_insert = rhs.user_defined_item;
                update_into_insert.extend(self.user_defined_item);
                update_into_insert
            },
            increments: {
                let mut update_into_insert = rhs.increments;
                update_into_insert.extend(self.increments);
                update_into_insert
            },
        }
    }
}

impl InternalNodeChanges {
    pub fn with(self, other: Self) -> Result<Self, PossibleChangesInternalError> {
        match (self, other) {
            (Self::Insert(_), Self::Insert(_)) => Err(PossibleChangesInternalError::MultipleInsertWithSameNode),
            (Self::Insert(_), Self::Delete(_)) | (Self::Delete(_), Self::Insert(_)) => {
                Err(PossibleChangesInternalError::InsertAndDelete)
            }
            (Self::Delete(_), Self::Delete(_)) => Err(PossibleChangesInternalError::MultipleDeleteWithSameNode),
            (Self::Insert(a), Self::Update(b)) => Ok(Self::Insert(a + b)),
            (Self::Update(a), Self::Insert(b)) => Ok(Self::Insert(a + b)),
            (Self::Update(a), Self::Update(b)) => Ok(Self::Update(a + b)),
            (Self::Update(_), Self::Delete(a)) | (Self::Delete(a), Self::Update(_)) => Ok(Self::Delete(a)),
        }
    }
}

impl Add<Self> for InsertRelationInternalInput {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            relation_names: {
                // TODO: shouldn't be empty
                let mut update_into_insert = rhs.relation_names;
                update_into_insert.extend(self.relation_names);
                update_into_insert
            },
            fields: {
                let mut update_into_insert = rhs.fields;
                update_into_insert.extend(self.fields);
                update_into_insert
            },
            ..rhs
        }
    }
}

impl Add<InsertRelationInternalInput> for UpdateRelationInternalInput {
    type Output = InsertRelationInternalInput;

    fn add(self, rhs: InsertRelationInternalInput) -> Self::Output {
        Self::Output {
            relation_names: {
                // TODO: shouldn't be empty
                let (a, b): (Vec<String>, Vec<String>) =
                    self.relation_names
                        .into_iter()
                        .partition_map(|relation| match relation {
                            UpdateRelation::Add(a) => itertools::Either::Right(a),
                            UpdateRelation::Remove(a) => itertools::Either::Left(a),
                        });
                let mut update_into_insert = rhs.relation_names;
                update_into_insert.extend(b);
                update_into_insert.into_iter().filter(|x| a.contains(x)).collect()
            },
            fields: {
                let mut update_into_insert = rhs.fields;
                update_into_insert.extend(self.user_defined_item);
                update_into_insert
            },
            ..rhs
        }
    }
}

impl Add<UpdateRelationInternalInput> for InsertRelationInternalInput {
    type Output = Self;

    fn add(self, rhs: UpdateRelationInternalInput) -> Self::Output {
        rhs + self
    }
}

impl Add<Self> for UpdateRelationInternalInput {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            relation_names: {
                // TODO: shouldn't be empty
                let mut update_into_insert = rhs.relation_names;
                update_into_insert.extend(self.relation_names);
                update_into_insert.into_iter().unique().collect()
            },
            user_defined_item: {
                let mut update_into_insert = rhs.user_defined_item;
                update_into_insert.extend(self.user_defined_item);
                update_into_insert
            },
            ..rhs
        }
    }
}

impl Add<DeleteMultipleRelationsInternalInput> for UpdateRelationInternalInput {
    type Output = Self;

    fn add(self, rhs: DeleteMultipleRelationsInternalInput) -> Self::Output {
        Self::Output {
            relation_names: {
                // TODO: shouldn't be empty
                let mut update_into_insert = self.relation_names;
                update_into_insert.extend(rhs.relation_names.into_iter().map(UpdateRelation::Remove));
                update_into_insert.into_iter().unique().collect()
            },
            user_defined_item: self.user_defined_item,
            ..self
        }
    }
}

impl Add<UpdateRelationInternalInput> for DeleteMultipleRelationsInternalInput {
    type Output = UpdateRelationInternalInput;

    fn add(self, rhs: UpdateRelationInternalInput) -> Self::Output {
        rhs + self
    }
}

impl Add<DeleteAllRelationsInternalInput> for UpdateRelationInternalInput {
    type Output = Self;

    fn add(self, rhs: DeleteAllRelationsInternalInput) -> Self::Output {
        Self::Output {
            relation_names: {
                // TODO: shouldn't be empty
                let mut update_into_insert = self.relation_names;
                update_into_insert.extend(
                    rhs.relation_names
                        .unwrap_or_default()
                        .into_iter()
                        .map(UpdateRelation::Remove),
                );
                update_into_insert.into_iter().unique().collect()
            },
            user_defined_item: self.user_defined_item,
            ..self
        }
    }
}

impl Add<UpdateRelationInternalInput> for DeleteAllRelationsInternalInput {
    type Output = UpdateRelationInternalInput;

    fn add(self, rhs: UpdateRelationInternalInput) -> Self::Output {
        rhs + self
    }
}

impl Add<Self> for DeleteRelationInternalInput {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Multiple(a), Self::Multiple(b)) => Self::Multiple(DeleteMultipleRelationsInternalInput {
                relation_names: {
                    let mut update_into_insert = a.relation_names;
                    update_into_insert.extend(b.relation_names);
                    update_into_insert.into_iter().unique().collect()
                },
                ..a
            }),
            (Self::Multiple(_), Self::All(a)) | (Self::All(a), Self::Multiple(_) | Self::All(_)) => Self::All(a),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum InternalRelationChanges {
    Insert(InsertRelationInternalInput), // One affected node
    Update(UpdateRelationInternalInput), // Unknown affected ids
    Delete(DeleteRelationInternalInput), // Unknown affected ids
}

impl InternalRelationChanges {
    pub fn with(self, other: Self) -> Result<Self, PossibleChangesInternalError> {
        match (self, other) {
            (Self::Insert(a), Self::Insert(b)) => Ok(Self::Insert(a + b)),
            (Self::Insert(_), Self::Delete(_)) | (Self::Delete(_), Self::Insert(_)) => {
                Err(PossibleChangesInternalError::InsertAndDelete)
            }
            (Self::Delete(a), Self::Delete(b)) => Ok(Self::Delete(a + b)),
            (Self::Insert(a), Self::Update(b)) => Ok(Self::Insert(a + b)),
            (Self::Update(a), Self::Insert(b)) => Ok(Self::Insert(a + b)),
            (Self::Update(a), Self::Update(b)) => Ok(Self::Update(a + b)),
            (Self::Update(b), Self::Delete(DeleteRelationInternalInput::All(a)))
            | (Self::Delete(DeleteRelationInternalInput::All(a)), Self::Update(b)) => Ok(Self::Update(a + b)),
            (Self::Update(b), Self::Delete(DeleteRelationInternalInput::Multiple(a)))
            | (Self::Delete(DeleteRelationInternalInput::Multiple(a)), Self::Update(b)) => Ok(Self::Update(a + b)),
        }
    }
}

impl InternalChanges {
    pub fn with(self, other: Self) -> Result<Self, PossibleChangesInternalError> {
        match (self, other) {
            (Self::Node(_) | Self::NodeConstraints(_), Self::Relation(_))
            | (Self::Node(_) | Self::Relation(_), Self::NodeConstraints(_))
            | (Self::NodeConstraints(_) | Self::Relation(_), Self::Node(_)) => {
                Err(PossibleChangesInternalError::NodeAndRelationCompare)
            }
            (Self::Relation(a), Self::Relation(b)) => a.with(b).map(Self::Relation),
            (Self::Node(a), Self::Node(b)) => a.with(b).map(Self::Node),
            (Self::NodeConstraints(a), Self::NodeConstraints(b)) => a.with(b).map(Self::NodeConstraints),
        }
    }
}

impl InternalNodeConstraintChanges {
    pub fn with(self, other: Self) -> Result<Self, PossibleChangesInternalError> {
        match (self, other) {
            // these can't happen for unique constraints
            (
                Self::Insert(InsertNodeConstraintInternalInput::Unique(left)),
                Self::Insert(InsertNodeConstraintInternalInput::Unique(_right)),
            ) => Ok(Self::Insert(InsertNodeConstraintInternalInput::Unique(left))),
            (
                Self::Update(UpdateNodeConstraintInternalInput::Unique(left)),
                Self::Update(UpdateNodeConstraintInternalInput::Unique(_right)),
            ) => Ok(Self::Update(UpdateNodeConstraintInternalInput::Unique(left))),
            (
                Self::Insert(InsertNodeConstraintInternalInput::Unique(insert)),
                Self::Update(UpdateNodeConstraintInternalInput::Unique(_update)),
            ) => Ok(Self::Insert(InsertNodeConstraintInternalInput::Unique(insert))),
            (
                Self::Update(UpdateNodeConstraintInternalInput::Unique(_update)),
                Self::Insert(InsertNodeConstraintInternalInput::Unique(insert)),
            ) => Ok(Self::Insert(InsertNodeConstraintInternalInput::Unique(insert))),
            // TODO: Need to add addition
            (Self::Delete(left), Self::Delete(_right)) => Ok(Self::Delete(left)),
            (Self::Insert(_) | Self::Update(_), Self::Delete(_))
            | (Self::Delete(_), Self::Insert(_) | Self::Update(_)) => unreachable!(),
        }
    }
}

#[cfg(not(feature = "local"))]
async fn execute(
    batchers: &'_ DynamoDBBatchersData,
    ctx: &'_ DynamoDBContext,
    changes: Vec<PossibleChanges>,
) -> Result<HashMap<TxItem, AttributeValue>, ToTransactionError> {
    info!(ctx.trace_id, "Public");
    for r in &changes {
        info!(ctx.trace_id, "{:?}", r);
    }

    // First step, we convert public change to our private interface
    let selections: Vec<_> = changes
        .into_iter()
        .map(|change| Box::pin(change.to_changes(batchers, ctx)))
        .collect();

    let selection_len = selections.len();
    // First await to select everything that'll change.
    let result = futures_util::future::try_join_all(selections).await?;

    info!(ctx.trace_id, "Private");
    for r in &result {
        for ((pk, sk), val) in r {
            info!(ctx.trace_id, "{} {} | {:?}", pk, sk, val);
        }
    }

    // Merge Hashmap together
    let merged: HashMap<TuplePartitionKeySortingKey, Vec<InternalChanges>> =
        result
            .into_iter()
            .fold(HashMap::with_capacity(selection_len), |mut acc, cur| {
                cur.into_iter().for_each(|(k, v)| match acc.entry(k) {
                    Entry::Vacant(vac) => {
                        vac.insert(vec![v]);
                    }
                    Entry::Occupied(mut oqp) => {
                        oqp.get_mut().push(v);
                    }
                });
                acc
            });

    // When every PossibleChanges are merged together, we do apply our merge of
    // possible_changes for each ID to create a TransactWriteItem
    let transactions: Vec<ToTransactionFuture<'_>> = merged
        .into_iter()
        .map(|((pk, sk), val)| val.to_transaction(batchers, ctx, pk, sk))
        .collect();

    let transactions_len = transactions.len();
    let transactions = futures_util::future::try_join_all(transactions).await?;

    let merged: HashMap<TxItem, AttributeValue> =
        transactions
            .into_iter()
            .fold(HashMap::with_capacity(transactions_len), |mut acc, cur| {
                acc.extend(cur);
                acc
            });

    Ok(merged)
}

#[cfg(feature = "local")]
async fn execute(
    batchers: &'_ DynamoDBBatchersData,
    ctx: &'_ DynamoDBContext,
    changes: Vec<PossibleChanges>,
) -> Result<Vec<crate::local::types::Operation>, ToTransactionError> {
    use crate::local::types::Operation;

    info!(ctx.trace_id, "Public");
    for r in &changes {
        info!(ctx.trace_id, "{:?}", r);
    }

    // First step, we convert public change to our private interface
    let selections: Vec<_> = changes
        .into_iter()
        .map(|change| Box::pin(change.to_changes(batchers, ctx)))
        .collect();

    let selection_len = selections.len();
    // First await to select everything that'll change.
    let result = futures_util::future::try_join_all(selections).await?;

    info!(ctx.trace_id, "Private");
    for r in &result {
        for ((pk, sk), val) in r {
            info!(ctx.trace_id, "{} {} | {:?}", pk, sk, val);
        }
    }

    // Merge Hashmap together
    let merged: HashMap<TuplePartitionKeySortingKey, Vec<InternalChanges>> =
        result
            .into_iter()
            .fold(HashMap::with_capacity(selection_len), |mut acc, cur| {
                cur.into_iter().for_each(|(k, v)| match acc.entry(k) {
                    Entry::Vacant(vac) => {
                        vac.insert(vec![v]);
                    }
                    Entry::Occupied(mut oqp) => {
                        oqp.get_mut().push(v);
                    }
                });
                acc
            });

    // When every PossibleChanges are merged together, we do apply our merge of
    // possible_changes for each ID to create a TransactWriteItem
    let transactions = merged
        .into_iter()
        .map(|((pk, sk), val)| val.to_transaction(batchers, ctx, pk, sk))
        .collect::<Vec<ToTransactionFuture<'_>>>();

    let transactions = futures_util::future::try_join_all(transactions).await?;

    let merged = transactions
        .into_iter()
        .map(|(sql, values, kind)| Operation { sql, values, kind })
        .collect();

    Ok(merged)
}

/// The result is not accessible, the Hashmap will be empty
async fn load_keys(
    batcher: &DynamoDBBatchersData,
    ctx: &DynamoDBContext,
    tx: Vec<PossibleChanges>,
    #[cfg(feature = "local")] local_ctx: &LocalContext,
) -> Result<HashMap<PossibleChanges, AttributeValue>, ToTransactionError> {
    info!(ctx.trace_id, "Execute");
    let mut result = HashMap::with_capacity(tx.len());
    for x in &tx {
        result.insert(x.clone(), AttributeValue { ..Default::default() });
    }

    #[cfg(not(feature = "local"))]
    let _a = execute(batcher, ctx, tx).await?;

    #[cfg(feature = "local")]
    {
        use crate::local::types::Constraint;
        use bridge_api::{mutation, ApiErrorKind, MutationError};

        let operations = execute(batcher, ctx, tx).await?;

        if !operations.is_empty() {
            mutation(operations, &local_ctx.bridge_port)
                .await
                .map_err(|error| match error {
                    MutationError::Surf(_) | MutationError::InternalServerError => ToTransactionError::Unknown,
                    MutationError::Api(api_error) => match api_error.error_kind {
                        ApiErrorKind::ConstraintViolation(Constraint::Unique { values, fields }) => {
                            ToTransactionError::UniqueCondition {
                                values,
                                fields,
                                source: TransactionError::UnknownError,
                            }
                        }
                    },
                })?;
        }
    }

    info!(ctx.trace_id, "Executed");
    Ok(result)
}

pub struct NewTransactionLoader {
    ctx: Arc<DynamoDBContext>,
    parent_ctx: Weak<DynamoDBBatchersData>,
    #[cfg(feature = "local")]
    local_ctx: Arc<LocalContext>,
}

#[async_trait::async_trait]
impl Loader<PossibleChanges> for NewTransactionLoader {
    type Value = AttributeValue;
    type Error = ToTransactionError;

    async fn load(&self, keys: &[PossibleChanges]) -> Result<HashMap<PossibleChanges, Self::Value>, Self::Error> {
        load_keys(
            &self.parent_ctx.upgrade().expect("can't fail"),
            &self.ctx,
            keys.to_vec(),
            #[cfg(feature = "local")]
            &self.local_ctx,
        )
        .await
    }
}

pub fn get_loader_transaction_new(
    ctx: Arc<DynamoDBContext>,
    parent_ctx: Weak<DynamoDBBatchersData>,
    #[cfg(feature = "local")] local_ctx: Arc<LocalContext>,
) -> DataLoader<NewTransactionLoader, LruCache> {
    DataLoader::with_cache(
        NewTransactionLoader {
            ctx,
            parent_ctx,
            #[cfg(feature = "local")]
            local_ctx,
        },
        |f| Runtime::locate().spawn(f),
        LruCache::new(128),
    )
    .max_batch_size(1024)
    .delay(Duration::from_millis(5))
}

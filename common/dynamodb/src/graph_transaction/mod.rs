use crate::dataloader::{DataLoader, Loader, LruCache};
use crate::TxItem;
use crate::{constant, QueryKey};
use crate::{BatchGetItemLoaderError, TransactionError};
use crate::{DynamoDBBatchersData, DynamoDBContext};
use chrono::Utc;
use derivative::Derivative;
use dynomite::AttributeValue;
use futures::Future;
use futures_util::TryFutureExt;
use itertools::Itertools;
use log::info;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Add;
use std::pin::Pin;
use std::sync::{Arc, Weak};
use std::time::Duration;

mod dynamodb;

type TupplePartitionKeySortingKey = (String, String);

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct InsertNodeInput {
    id: String,
    ty: String,
    #[derivative(Debug = "ignore")]
    user_defined_item: HashMap<String, AttributeValue>,
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
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct LinkNodeCachedInput {
    from_id: String,
    from_ty: String,
    to_id: String,
    to_ty: String,
    relation_name: String,
    #[derivative(Debug = "ignore")]
    user_defined_item: HashMap<String, AttributeValue>,
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
    pub const fn new_node(ty: String, id: String, user_defined_item: HashMap<String, AttributeValue>) -> Self {
        Self::InsertNode(InsertNodeInput {
            id,
            ty,
            user_defined_item,
        })
    }

    pub const fn update_node(ty: String, id: String, user_defined_item: HashMap<String, AttributeValue>) -> Self {
        Self::UpdateNode(UpdateNodeInput {
            id,
            ty,
            user_defined_item,
        })
    }

    pub const fn delete_node(ty: String, id: String) -> Self {
        Self::DeleteNode(DeleteNodeInput { id, ty })
    }

    pub const fn new_link_cached(
        from_ty: String,
        from_id: String,
        to_ty: String,
        to_id: String,
        relation_name: String,
        user_defined_item: HashMap<String, AttributeValue>,
    ) -> Self {
        Self::LinkRelation(LinkNodeInput::Cache(LinkNodeCachedInput {
            from_id,
            from_ty,
            to_id,
            to_ty,
            relation_name,
            user_defined_item,
        }))
    }

    pub const fn unlink_node(
        from_ty: String,
        from_id: String,
        to_ty: String,
        to_id: String,
        relation_name: String,
    ) -> Self {
        Self::UnlinkRelation(UnlinkNodeInput {
            from_id,
            from_ty,
            to_id,
            to_ty,
            relation_name,
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
    fn to_changes<'a>(self, _batchers: &'a DynamoDBBatchersData, _ctx: &'a DynamoDBContext) -> SelectionType<'a> {
        let pk = format!("{}#{}", &self.ty, &self.id);

        Box::pin(async {
            Ok(HashMap::from([(
                (pk.clone(), pk),
                InternalChanges::Node(InternalNodeChanges::Insert(InsertNodeInternalInput {
                    id: self.id,
                    ty: self.ty,
                    user_defined_item: self.user_defined_item,
                })),
            )]))
        })
    }
}

impl GetIds for UpdateNodeInput {
    fn to_changes<'a>(self, batchers: &'a DynamoDBBatchersData, ctx: &'a DynamoDBContext) -> SelectionType<'a> {
        let pk = format!("{}#{}", &self.ty, &self.id);

        let query_loader_reversed = &batchers.query_reversed;
        let select_entities_to_update = query_loader_reversed
            .load_one(QueryKey::new(pk, Vec::new()))
            .map_ok(|x| {
                std::iter::once(x)
                    .into_iter()
                    .flatten()
                    .flat_map(|x| {
                        x.values.into_iter().flat_map(|(_, y)| {
                            y.node
                                .into_iter()
                                .chain(y.edges.into_iter().flat_map(|(_, val)| val.into_iter()))
                        })
                    })
                    .filter_map(|mut x| {
                        let pk = x.remove("__pk").and_then(|x| x.s);
                        let sk = x.remove("__sk").and_then(|x| x.s);

                        match (pk, sk) {
                            (Some(pk), Some(sk)) => Some((pk, sk)),
                            _ => None,
                        }
                    })
                    .collect::<Vec<(String, String)>>()
            });

        Box::pin(async move {
            let ids = select_entities_to_update
                .await
                .map_err(|_| BatchGetItemLoaderError::UnknownError)?;

            let id_len = ids.len() + 1;
            let mut result = HashMap::with_capacity(id_len);

            for (pk, sk) in ids {
                info!(ctx.trace_id, "Asking for update of {} {}", &pk, &sk);
                let (from_ty, from_id) = pk.rsplit_once('#').ok_or(BatchGetItemLoaderError::UnknownError)?;
                let (to_ty, to_id) = sk.rsplit_once('#').ok_or(BatchGetItemLoaderError::UnknownError)?;

                let from_ty = from_ty.to_owned();
                let from_id = from_id.to_owned();
                let to_ty = to_ty.to_owned();
                let to_id = to_id.to_owned();

                if pk == sk {
                    result.insert(
                        (pk, sk),
                        InternalChanges::Node(InternalNodeChanges::Update(UpdateNodeInternalInput {
                            id: from_id,
                            ty: from_ty,
                            user_defined_item: self.user_defined_item.clone(),
                        })),
                    );
                } else {
                    result.insert(
                        (pk, sk),
                        InternalChanges::Relation(InternalRelationChanges::Update(UpdateRelationInternalInput {
                            from_id,
                            from_ty,
                            to_ty,
                            to_id,
                            user_defined_item: self.user_defined_item.clone(),
                            relation_names: Vec::new(),
                        })),
                    );
                }
            }

            Ok(result)
        })
    }
}

impl GetIds for DeleteNodeInput {
    fn to_changes<'a>(self, batchers: &'a DynamoDBBatchersData, ctx: &'a DynamoDBContext) -> SelectionType<'a> {
        let id_to_be_deleted = format!("{}#{}", &self.ty, &self.id);
        let query_loader = &batchers.query;
        let query_loader_reversed = &batchers.query_reversed;

        let items_pk = query_loader.load_one(QueryKey::new(id_to_be_deleted.clone(), Vec::new()));
        let items_sk = query_loader_reversed.load_one(QueryKey::new(id_to_be_deleted.clone(), Vec::new()));

        let items_to_be_deleted = futures_util::future::try_join_all(vec![items_pk, items_sk]).map_ok(|x| {
            x.into_iter()
                .flatten()
                .flat_map(|x| {
                    x.values.into_iter().flat_map(|(_, y)| {
                        y.node
                            .into_iter()
                            .chain(y.edges.into_iter().flat_map(|(_, val)| val.into_iter()))
                    })
                })
                .filter_map(|mut x| {
                    let pk = x.remove("__pk").and_then(|x| x.s);
                    let sk = x.remove("__sk").and_then(|x| x.s);

                    match (pk, sk) {
                        (Some(pk), Some(sk)) => Some((pk, sk)),
                        _ => None,
                    }
                })
                .collect::<Vec<(String, String)>>()
        });

        // To remove a Node, we Remove the node and every relations (as the node is deleted)
        Box::pin(async {
            let ids = items_to_be_deleted
                .await
                .map_err(|_| BatchGetItemLoaderError::UnknownError)?;

            let id_len = ids.len() + 1;
            let mut result = HashMap::with_capacity(id_len);
            result.insert(
                (id_to_be_deleted.clone(), id_to_be_deleted),
                InternalChanges::Node(InternalNodeChanges::Delete(DeleteNodeInternalInput {
                    id: self.id,
                    ty: self.ty,
                })),
            );

            for (pk, sk) in ids.into_iter().filter(|(pk, sk)| pk != sk) {
                info!(ctx.trace_id, "{} {}", &pk, &sk);
                let (from_ty, from_id) = pk.rsplit_once('#').ok_or(BatchGetItemLoaderError::UnknownError)?;
                let (to_ty, to_id) = sk.rsplit_once('#').ok_or(BatchGetItemLoaderError::UnknownError)?;

                let from_ty = from_ty.to_owned();
                let from_id = from_id.to_owned();
                let to_ty = to_ty.to_owned();
                let to_id = to_id.to_owned();

                result.insert(
                    (pk, sk),
                    InternalChanges::Relation(InternalRelationChanges::Delete(DeleteRelationInternalInput::All(
                        DeleteAllRelationsInternalInput {
                            from_id,
                            from_ty,
                            to_id,
                            to_ty,
                            relation_names: None,
                        },
                    ))),
                );
            }

            Ok(result)
        })
    }
}

impl GetIds for LinkNodeCachedInput {
    fn to_changes<'a>(self, _batchers: &'a DynamoDBBatchersData, _ctx: &'a DynamoDBContext) -> SelectionType<'a> {
        let pk = format!("{}#{}", &self.from_ty, &self.from_id);
        let sk = format!("{}#{}", &self.to_ty, &self.to_id);
        Box::pin(async {
            Ok(HashMap::from([(
                (pk, sk),
                InternalChanges::Relation(InternalRelationChanges::Insert(InsertRelationInternalInput {
                    from_id: self.from_id,
                    from_ty: self.from_ty,
                    to_id: self.to_id,
                    to_ty: self.to_ty,
                    relation_names: vec![self.relation_name],
                    fields: self.user_defined_item,
                })),
            )]))
        })
    }
}

impl GetIds for LinkNodeNoCacheInput {
    fn to_changes<'a>(self, batchers: &'a DynamoDBBatchersData, _ctx: &'a DynamoDBContext) -> SelectionType<'a> {
        let pk = format!("{}#{}", &self.from_ty, &self.from_id);
        let sk = format!("{}#{}", &self.to_ty, &self.to_id);
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
        let pk = format!("{}#{}", &self.from_ty, &self.from_id);
        let sk = format!("{}#{}", &self.to_ty, &self.to_id);
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

type ToTransactionFuture<'a> =
    Pin<Box<dyn Future<Output = Result<HashMap<TxItem, AttributeValue>, ToTransactionError>> + Send + 'a>>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ToTransactionError {
    #[error("Internal error")]
    Unknown,
    #[error("Database internal error")]
    GetItemError(#[from] BatchGetItemLoaderError),
    #[error("{0}")]
    TransactionError(#[from] TransactionError),
}

trait ExecuteChangesOnDatabase
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

#[derive(Derivative, PartialEq, Clone)]
#[derivative(Debug)]
struct InsertNodeInternalInput {
    id: String,
    ty: String,
    #[derivative(Debug = "ignore")]
    user_defined_item: HashMap<String, AttributeValue>,
}

#[derive(Derivative, PartialEq, Clone)]
#[derivative(Debug)]
struct UpdateNodeInternalInput {
    id: String,
    ty: String,
    #[derivative(Debug = "ignore")]
    user_defined_item: HashMap<String, AttributeValue>,
}

impl UpdateNodeInternalInput {
    fn to_update_expression(
        values: HashMap<String, AttributeValue>,
        exp_values: &mut HashMap<String, AttributeValue>,
        exp_names: &mut HashMap<String, String>,
    ) -> String {
        let update_expression = values
            .into_iter()
            .filter(|(name, _)| !name.starts_with("__"))
            .chain(std::iter::once((
                constant::UPDATED_AT.to_string(),
                AttributeValue {
                    s: Some(Utc::now().to_string()),
                    ..Default::default()
                },
            )))
            .unique_by(|(name, _)| name.to_string())
            .map(|(name, value)| {
                let idx = format!(":{}", name.as_str());
                let sanitized_name = format!("#{}", name.as_str());
                let result = format!("{}={}", sanitized_name, idx);
                exp_values.insert(idx, value);
                exp_names.insert(sanitized_name, name.as_str().to_string());
                result
            })
            .join(",");

        format!("set {update_expression}")
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct DeleteNodeInternalInput {
    id: String,
    ty: String,
}

#[derive(Derivative, PartialEq, Clone)]
#[derivative(Debug)]
struct InsertRelationInternalInput {
    from_id: String,
    from_ty: String,
    to_id: String,
    to_ty: String,
    relation_names: Vec<String>,
    /// Those fields are not user_defined_item, privates are here too.
    #[derivative(Debug = "ignore")]
    fields: HashMap<String, AttributeValue>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum DeleteRelationInternalInput {
    Multiple(DeleteMultipleRelationsInternalInput),
    All(DeleteAllRelationsInternalInput),
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct DeleteAllRelationsInternalInput {
    from_id: String,
    from_ty: String,
    to_id: String,
    to_ty: String,
    /// Used to specify which relation_names are deleted, used for Update addition
    /// If not there, the delete will still happen
    relation_names: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct DeleteMultipleRelationsInternalInput {
    from_id: String,
    from_ty: String,
    to_id: String,
    to_ty: String,
    relation_names: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum UpdateRelation {
    #[allow(unused)]
    Add(String),
    Remove(String),
}
#[derive(Derivative, PartialEq, Clone)]
#[derivative(Debug)]
struct UpdateRelationInternalInput {
    from_id: String,
    from_ty: String,
    to_id: String,
    to_ty: String,
    relation_names: Vec<UpdateRelation>,
    #[derivative(Debug = "ignore")]
    user_defined_item: HashMap<String, AttributeValue>,
}

impl UpdateRelationInternalInput {
    fn to_update_expression(
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
                    AttributeValue {
                        s: Some(Utc::now().to_string()),
                        ..Default::default()
                    },
                )))
                .unique_by(|(name, _)| name.to_string())
                .map(|(name, value)| {
                    let idx = format!(":{}", name.as_str());
                    let sanitized_name = format!("#{}", name.as_str());
                    let result = format!("{}={}", sanitized_name, idx);
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
            exp_names.insert("#relation_names".to_string(), "__relation_names".to_string());
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
enum InternalNodeChanges {
    Insert(InsertNodeInternalInput),
    Update(UpdateNodeInternalInput), // Unknow affected ids
    Delete(DeleteNodeInternalInput), // Unknow affected ids
}

/// Private interface
#[derive(Debug, PartialEq, Clone)]
enum InternalChanges {
    Node(InternalNodeChanges),
    Relation(InternalRelationChanges),
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
            from_id: rhs.from_id,
            from_ty: rhs.from_ty,
            to_id: rhs.to_id,
            to_ty: rhs.to_ty,
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
        }
    }
}

impl Add<InsertRelationInternalInput> for UpdateRelationInternalInput {
    type Output = InsertRelationInternalInput;

    fn add(self, rhs: InsertRelationInternalInput) -> Self::Output {
        Self::Output {
            from_id: rhs.from_id,
            from_ty: rhs.from_ty,
            to_id: rhs.to_id,
            to_ty: rhs.to_ty,
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
            from_id: rhs.from_id,
            from_ty: rhs.from_ty,
            to_id: rhs.to_id,
            to_ty: rhs.to_ty,
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
        }
    }
}

impl Add<DeleteMultipleRelationsInternalInput> for UpdateRelationInternalInput {
    type Output = Self;

    fn add(self, rhs: DeleteMultipleRelationsInternalInput) -> Self::Output {
        Self::Output {
            from_id: self.from_id,
            from_ty: self.from_ty,
            to_id: self.to_id,
            to_ty: self.to_ty,
            relation_names: {
                // TODO: shouldn't be empty
                let mut update_into_insert = self.relation_names;
                update_into_insert.extend(rhs.relation_names.into_iter().map(UpdateRelation::Remove));
                update_into_insert.into_iter().unique().collect()
            },
            user_defined_item: self.user_defined_item,
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
            from_id: self.from_id,
            from_ty: self.from_ty,
            to_id: self.to_id,
            to_ty: self.to_ty,
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
                from_ty: a.from_ty,
                from_id: a.from_id,
                to_ty: a.to_ty,
                to_id: a.to_id,
                relation_names: {
                    let mut update_into_insert = a.relation_names;
                    update_into_insert.extend(b.relation_names);
                    update_into_insert.into_iter().unique().collect()
                },
            }),
            (Self::Multiple(_), Self::All(a)) | (Self::All(a), Self::Multiple(_) | Self::All(_)) => Self::All(a),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum InternalRelationChanges {
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
            (Self::Node(a), Self::Node(b)) => a.with(b).map(Self::Node),
            (Self::Node(_), Self::Relation(_)) | (Self::Relation(_), Self::Node(_)) => {
                Err(PossibleChangesInternalError::NodeAndRelationCompare)
            }
            (Self::Relation(a), Self::Relation(b)) => a.with(b).map(Self::Relation),
        }
    }
}

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
    let merged: HashMap<TupplePartitionKeySortingKey, Vec<InternalChanges>> =
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

/// The result is not accessible, the Hashmap will be empty
async fn load_keys(
    batcher: &DynamoDBBatchersData,
    ctx: &DynamoDBContext,
    tx: Vec<PossibleChanges>,
) -> Result<HashMap<PossibleChanges, AttributeValue>, ToTransactionError> {
    info!(ctx.trace_id, "Execute");
    let mut result = HashMap::with_capacity(tx.len());
    for x in &tx {
        result.insert(x.clone(), AttributeValue { ..Default::default() });
    }

    let _a = execute(batcher, ctx, tx).await?;
    info!(ctx.trace_id, "Executed");
    Ok(result)
}

pub struct NewTransactionLoader {
    ctx: Arc<DynamoDBContext>,
    parent_ctx: Weak<DynamoDBBatchersData>,
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
        )
        .await
    }
}

pub fn get_loader_transaction_new(
    ctx: Arc<DynamoDBContext>,
    parent_ctx: Weak<DynamoDBBatchersData>,
) -> DataLoader<NewTransactionLoader, LruCache> {
    DataLoader::with_cache(
        NewTransactionLoader { ctx, parent_ctx },
        wasm_bindgen_futures::spawn_local,
        LruCache::new(128),
    )
    .max_batch_size(1024)
    .delay(Duration::from_millis(5))
}

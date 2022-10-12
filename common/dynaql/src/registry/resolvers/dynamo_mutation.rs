use super::{ResolvedValue, ResolverContext, ResolverTrait};
use crate::registry::utils::{type_to_base_type, value_to_attribute};
use crate::registry::variables::id::ObfuscatedID;
use crate::registry::variables::VariableResolveDefinition;
use crate::registry::MetaType;
use crate::{Context, Error, ServerError, Value};
use chrono::{SecondsFormat, Utc};
use dynamodb::graph_transaction::PossibleChanges;
use dynamodb::model::node::NodeID;
use dynamodb::{BatchGetItemLoaderError, DynamoDBBatchersData, QueryKey, TransactionError};
use dynaql_value::Name;
use dynomite::{Attribute, AttributeValue};
use futures_util::future::Shared;
use futures_util::{FutureExt, StreamExt, TryFutureExt};
use indexmap::IndexMap;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::ops::Add;
use std::pin::Pin;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use ulid_rs::Ulid;

#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum DynamoMutationResolver {
    /// Create a new Node
    ///
    /// We do create a new node and store the generated ID into the ResolverContext to allow a
    /// ContextData Resolver to access this id if needed.
    /// When we create a Node with Edges, we fetch those edges before creating the Node and the
    /// vertices.
    ///
    /// # Flow
    ///
    /// -> Generate the ID of the new Node
    /// -> Fetch the Edges needed.
    /// -> Store the Node
    /// -> Store the Vertices.
    ///
    /// # Returns
    ///
    /// This resolver return a Value like this:
    ///
    /// ```json
    /// {
    ///   "id": "<generated_id>"
    /// }
    /// ```
    CreateNode {
        input: VariableResolveDefinition,
        /// Type defined for GraphQL side, it's used to be able to know if we manipulate a Node
        /// and if this Node got Edges. This type must the the Type visible on the GraphQL Schema.
        ty: String,
    },
    /// The delete Node will delete the node and the relation to the associated edges, as
    /// an edge is a Node, we won't have any unreachable node in our Database.
    ///
    /// We also store the deleted ID into the ResolverContext to allow a ContextData Resolver to
    /// access this id if needed.
    ///
    /// # Example
    ///
    /// A node with two edges:
    ///
    /// ```ignore
    ///                     ┌────────┐
    ///                 ┌───┤ Edge 1 │
    ///                 │   └────────┘
    ///      ┌────┐     │
    ///      │Node├─────┤
    ///      └────┘     │
    ///                 │   ┌────────┐
    ///                 └───┤ Edge 2 │
    ///                     └────────┘
    /// ```
    ///
    /// When we delete this node, we'll update the graph to become:
    ///
    /// ```ignore
    ///                     ┌────────┐
    ///                     │ Edge 1 │
    ///                     └────────┘
    ///                     
    ///
    ///
    ///                     ┌────────┐
    ///                     │ Edge 2 │
    ///                     └────────┘
    /// ```
    ///
    /// And as every edges of a Node are a Node too, they are still reachable.
    ///
    /// In the future, when we'll have worked on an async process to optimize we'll be able to
    /// optimize the delete operation:
    ///
    /// In fact it's useless to delete the vertices between the node when you do not have a
    /// bi-directional relaton between nodes. You could only remove the node and have an async
    /// process remove the vertices as soon as possible. It woulnd't affect the future user's
    /// queries but would allow a deletion to be executed with a constant time of one operation.
    ///
    /// # Returns
    ///
    /// This resolver return a Value like this:
    ///
    /// ```json
    /// {
    ///   "id": "<deleted_id>"
    /// }
    /// ```
    DeleteNode {
        id: VariableResolveDefinition,
        /// Type defined for GraphQL side, it's used to be able to know if we manipulate a Node
        /// and if this Node got Edges. This type must the the Type visible on the GraphQL Schema.
        ty: String,
    },
    /// Update a Node and related relations
    ///
    /// To update a Node, we need to fetch every duplicate of this node which will
    /// exists linked to other nodes.
    ///
    /// Trigger the update for those basic fields accross every node & duplicate.
    ///
    /// ```json
    /// {
    ///   "id": "<updated_id>"
    /// }
    /// ```
    UpdateNode {
        id: VariableResolveDefinition,
        input: VariableResolveDefinition,
        /// Type defined for GraphQL side, it's used to be able to know if we manipulate a Node
        /// and if this Node got Edges. This type must the the Type visible on the GraphQL Schema.
        ty: String,
    },
}

type SharedSelectionType<'a> = Shared<
    Pin<
        Box<
            dyn Future<
                    Output = Result<
                        HashMap<(String, String), HashMap<String, AttributeValue>>,
                        BatchGetItemLoaderError,
                    >,
                > + Send
                + 'a,
        >,
    >,
>;

type SelectionType<'a> = Pin<
    Box<
        dyn Future<
                Output = Result<
                    HashMap<(String, String), HashMap<String, AttributeValue>>,
                    BatchGetItemLoaderError,
                >,
            > + Send
            + 'a,
    >,
>;

type TransactionType<'a> = Pin<Box<dyn Future<Output = Result<ResolvedValue, Error>> + Send + 'a>>;

/// The purpose of this struct is to divide result based on transaction or selection.
/// And these results will be based on a projection of what would exist if we executed
/// the creation and the selection.
///
/// Currently the transaction mechanism we came up with is based on a DataLoader pattern
/// due to DynamoDB inner working which doesn't grant a proper Transaction System
/// where we can begin at the start of the request and at the end if there is an
/// issue, we revert everything.
/// This block will need to be created later.
///
/// As it's implemented following a DataLoader pattern, it's optimized to avoid
/// having too many requests, but it means we should have every modification bach
/// together.
///
/// The idea is to properly work on transaction after the main features for Gateway
/// are done.
struct RecursiveCreation<'a> {
    /// Projected + Actual Selection
    /// If we want to create an Entity with a Edge, the Projected Edge would be
    /// return here for instance, and the real future to create it would be inside
    /// the `transaction`
    pub selection: SharedSelectionType<'a>,
    pub transaction: Vec<TransactionType<'a>>,
}

impl<'a> Add<RecursiveCreation<'a>> for RecursiveCreation<'a> {
    type Output = RecursiveCreation<'a>;
    fn add(self, rhs: RecursiveCreation<'a>) -> Self::Output {
        let selection_future: SelectionType = Box::pin(async move {
            let select = futures_util::future::try_join_all(vec![rhs.selection, self.selection])
                .map_ok(|r| {
                    r.into_iter().reduce(|mut acc, curr| {
                        acc.extend(curr);
                        acc
                    })
                })
                .await?
                .unwrap_or_default();

            Ok(select)
        });

        let selection_entity = selection_future.shared();
        let mut transactions = self.transaction;
        transactions.extend(rhs.transaction);

        Self {
            selection: selection_entity,
            transaction: transactions,
        }
    }
}

/// Create an Node and the relation associated to this Node if the Node is
/// modelized
///
/// To create a Node, we'll follow these steps,
/// For every relation:
///     - (Create the sub-node if we need to create it)
///     - Create The relation between the parent-node and the sub-node
/// Return:
///     - The projected data of the sub-node
///
/// For every relation:
///     - If the sub-node should have the reversed relation, create it
///
/// Then create the node
///
/// Return a flattened version of Vec<Future> for every transactions which will
/// need to be run.
fn node_create<'a>(
    ctx: &'a Context<'a>,
    node_ty: &'a MetaType,
    execution_id: Ulid,
    increment: Arc<AtomicUsize>,
    input: IndexMap<Name, Value>,
) -> RecursiveCreation<'a> {
    let current_execution_id = {
        let mut execution_id = Some(execution_id);
        for _ in 0..increment.fetch_add(1, std::sync::atomic::Ordering::SeqCst) {
            execution_id = execution_id.as_ref().and_then(Ulid::increment);
        }

        execution_id
    }
    .expect("Shouldn't fail");

    let id = NodeID::new_owned(node_ty.name().to_string(), current_execution_id.to_string());
    // First, to create the Node, we'll need to create the associated relations
    // if they need to be created.
    let relations_to_be_created = node_ty.relations();

    // We do copy every value from the input we do have into the item we'll
    // insert
    let item = input
        .clone()
        .into_iter()
        .filter(|(key, _)| !relations_to_be_created.contains_key(key.as_str()))
        .fold(HashMap::new(), |mut acc, (key, val)| {
            let key = key.to_string();
            acc.insert(
                key,
                value_to_attribute(val.into_json().expect("can't fail")),
            );
            acc
        });

    let cloned_item = item.clone();
    let id_cloned = id.clone();
    let selection_future: SelectionType = Box::pin(async move {
        let mut result = HashMap::with_capacity(1);
        result.insert((id_cloned.to_string(), id_cloned.to_string()), cloned_item);
        Ok(result)
    });
    let selection_entity = selection_future.shared();

    // The tricky part here, is, if we run them altogether, it means we'll create
    // edges, but if the mutation fail before the end we could have partial application
    // of the transaction.
    //
    // To solution this issue, we'll split them up in two part, creation, and selection.
    // We'll execute the selections future to check if the selected edges are compliant.
    // Then we'll run the transaction to create every node & relation, if it fails,
    // we'll at least have a rollback splitted by 25, so, even if it dosn't not fix
    // everything, we'll have a way to fix it in the future.
    let (_, mut transactions) = relations_to_be_created
        .into_iter()
        .map(|(field, relation)| {
            (
                &relation.name,
                relation_handle(
                    ctx,
                    node_ty,
                    selection_entity.clone(),
                    field,
                    &relation.name,
                    &input,
                    execution_id,
                    increment.clone(),
                ),
            )
        })
        .fold(
            (Vec::new(), Vec::new()),
            |(mut selections, mut transactions), (relation_name, list_recur)| {
                for curr in list_recur {
                    selections.extend(vec![curr
                        .selection
                        .map_ok(|val| (relation_name.clone(), val))]);
                    transactions.extend(curr.transaction.into_iter());
                }
                (selections, transactions)
            },
        );

    // Once we have the edges, either in the process of being created or created
    // we do have their id, so now, we need to:
    //   - Create the targeted Node
    let create_future: Pin<Box<dyn Future<Output = Result<ResolvedValue, Error>> + Send>> =
        Box::pin(async move {
            let batchers = ctx.data_unchecked::<Arc<DynamoDBBatchersData>>();
            let transaction_loader = &batchers.transaction_new;

            let node = PossibleChanges::new_node(
                node_ty.name().to_string(),
                current_execution_id.to_string(),
                item,
                node_ty
                    .constraints()
                    .iter()
                    .cloned()
                    .map(From::from)
                    .collect(),
            );

            transaction_loader
                .load_many(vec![node])
                .await
                .map_err(Error::new_with_source)?;

            Ok(ResolvedValue::new(Arc::new(serde_json::json!({
                "id": serde_json::Value::String(id.to_string()),
            }))))
        });

    transactions.extend(vec![create_future]);

    RecursiveCreation {
        selection: selection_entity,
        transaction: transactions,
    }
}

/// Delete a relation on both side if they exist for a relation name for one entity
///
/// The strategy is:
/// -> We get the ID1, ID2
/// So we get, the first relation & the second one
/// Then we remove those
async fn relation_remove<'a>(
    ctx: &'a Context<'a>,
    from: SharedSelectionType<'a>,
    to: String,
    relation_name: &'a str,
) -> Result<ResolvedValue, Error> {
    let batchers = ctx.data_unchecked::<Arc<DynamoDBBatchersData>>();
    let transaction_loader = &batchers.transaction_new;
    let values = from.await.map_err(Error::new_with_source)?;

    let mut transactions = Vec::with_capacity(values.len() * 2 + 1);

    for ((pk, _), _) in values
        .into_iter()
        .filter(|((pk, sk), _)| *pk != to || *sk != to)
    {
        let from = ObfuscatedID::new(&pk).map_err(Error::new_with_source)?;
        let to = ObfuscatedID::new(&to).map_err(Error::new_with_source)?;

        let from_to_to = PossibleChanges::unlink_node(
            from.ty().to_string(),
            from.id().to_string(),
            to.ty().to_string(),
            to.id().to_string(),
            relation_name.to_string(),
        );
        let to_to_from = PossibleChanges::unlink_node(
            to.ty().to_string(),
            to.id().to_string(),
            from.ty().to_string(),
            from.id().to_string(),
            relation_name.to_string(),
        );

        transactions.push(from_to_to);
        transactions.push(to_to_from);
    }

    transaction_loader
        .load_many(transactions)
        .await
        .map_err(Error::new_with_source)?;

    Ok(ResolvedValue::new(Arc::new(serde_json::Value::Null)))
}

type InputIterRef<'a> = Vec<(&'a Name, &'a Value)>;
type InputIter = Vec<(Name, Value)>;

/// Update a node
///
/// An update means:
///   - Updating basic fields for the entity and also for every duplicate linked
///   to this node.
///   - Create new linked entity if needed
///   - Remove old entity linked if needed
fn node_update<'a>(
    ctx: &'a Context<'a>,
    node_ty: &'a MetaType,
    execution_id: Ulid,
    increment: Arc<AtomicUsize>,
    input: IndexMap<Name, Value>,
    id: String,
) -> RecursiveCreation<'a> {
    let relations = node_ty.relations();

    let id_cloned = id.clone();
    let (_, basic): (InputIterRef<'_>, InputIterRef<'_>) = input
        .iter()
        .partition(|(name, _)| relations.contains_key(name.as_str()));
    let should_update_updated_at = !basic.is_empty();
    // We compute the attribute which will be updated.
    let update_attr: InputIter = basic
        .into_iter()
        .map(|(name, val)| (name.clone(), val.clone()))
        .collect();

    // We create an updated version of the selected entity
    let selection_updated_future: SelectionType = Box::pin(async move {
        let batchers = ctx.data_unchecked::<Arc<DynamoDBBatchersData>>();
        let loader = &batchers.loader;

        loader
            .load_many(vec![(id_cloned.clone(), id_cloned)])
            .await
            .map(|selected| {
                selected
                    .into_iter()
                    .map(|(id, mut entity)| {
                        for (att_name, att_val) in &update_attr {
                            entity.insert(
                                att_name.to_string(),
                                value_to_attribute(
                                    att_val
                                        .clone()
                                        .into_json()
                                        .expect("Shouldn't fail as this is valid json"),
                                ),
                            );
                        }
                        if should_update_updated_at {
                            entity.insert(
                                "updated_at".to_string(),
                                Utc::now()
                                    .to_rfc3339_opts(SecondsFormat::Millis, true)
                                    .into_attr(),
                            );
                        }
                        (id, entity)
                    })
                    .collect()
            })
    });
    let selection_entity_updated = selection_updated_future.shared();

    // We manage every relations possible
    let (_, mut transactions) = relations
        .clone()
        .into_iter()
        .map(|(field, relation)| {
            (
                &relation.name,
                relation_handle(
                    ctx,
                    node_ty,
                    selection_entity_updated.clone(),
                    field,
                    &relation.name,
                    &input,
                    execution_id,
                    increment.clone(),
                ),
            )
        })
        .fold(
            (Vec::new(), Vec::new()),
            |(mut selections, mut transactions), (relation_name, list_recur)| {
                for curr in list_recur {
                    selections.extend(vec![(
                        relation_name,
                        curr.selection.map_ok(|val| (relation_name.clone(), val)),
                    )]);
                    transactions.extend(curr.transaction.into_iter());
                }
                (selections, transactions)
            },
        );

    // We prepare a selection future which will be ran before any transaction
    // We'll execute the update even if the relation will be unlink after.
    // We can optimize this, but it's easier to have the same flow right now.
    let id_cloned = id.clone();
    let batchers = ctx.data_unchecked::<Arc<DynamoDBBatchersData>>();
    let query_loader_reversed = &batchers.query_reversed;
    let select_entities_to_update = query_loader_reversed
        .load_one(QueryKey::new(id, Vec::new()))
        .shared();

    let slection_cloned = selection_entity_updated.clone();
    // We create the update future which will be triggered after every selection future
    // to update the main node and also the replicate.
    // This future will also create/delete relation if needed and create node if needed.
    let update_future: Pin<Box<dyn Future<Output = Result<ResolvedValue, Error>> + Send>> =
        Box::pin(async move {
            let batchers = ctx.data_unchecked::<Arc<DynamoDBBatchersData>>();
            let transaction_batcher = &batchers.transaction_new;
            let selection = slection_cloned
                .clone()
                .await
                .map_err(Error::new_with_source)?
                .into_iter()
                .next()
                .map(|(_, x)| x)
                .ok_or(TransactionError::UnknownError)
                .map_err(Error::new_with_source)?;

            let from = ObfuscatedID::new(&id_cloned).map_err(Error::new_with_source)?;
            let update = PossibleChanges::update_node(
                from.ty().to_string(),
                from.id().to_string(),
                selection,
            );

            transaction_batcher
                .load_one(update)
                .await
                .map_err(Error::new_with_source)?;

            Ok(ResolvedValue::new(Arc::new(serde_json::Value::Null)))
        });

    transactions.extend(vec![update_future]);

    // We craft a selection future which will run the selection to get entities to
    // update so when the transaction run we prevent any possible race condition.
    let selected: SelectionType = Box::pin(async move {
        let (selection_entity, _) =
            futures_util::join!(selection_entity_updated, select_entities_to_update);

        selection_entity
    });
    let selected_shared = selected.shared();

    RecursiveCreation {
        selection: selected_shared,
        transaction: transactions,
    }
}

/// Get inputs list
fn inputs(parent_input: &Value) -> Option<Vec<&IndexMap<Name, Value>>> {
    match parent_input {
        Value::Object(obj) => Some(vec![obj]),
        Value::List(list) => {
            let input_list = list.iter().map(inputs).flatten().flatten().collect();
            Some(input_list)
        }
        _ => None,
    }
}

/// Create a relation node only if needed
async fn create_relation_node<'a>(
    ctx: &'a Context<'a>,
    to_ty: &MetaType,
    parent_value: SharedSelectionType<'a>,
    selected_value: SharedSelectionType<'a>,
    relation_name: &'a str,
) -> Result<ResolvedValue, Error> {
    let batchers = ctx.data_unchecked::<Arc<DynamoDBBatchersData>>();
    let transaction_batcher = &batchers.transaction_new;

    // Reverse Selected -> Parent
    let relation = to_ty
        .relations()
        .into_iter()
        .find(|(_, relation)| relation.name == relation_name);

    match relation {
        Some((_, relation)) => {
            // If we found the relation, it means we'll need a reverse
            // link.
            let ((from, _), parent_value) = &parent_value
                .await
                .map_err(Error::new_with_source)?
                .into_iter()
                .next()
                .ok_or(TransactionError::UnknownError)
                .map_err(Error::new_with_source)?;

            let from_ty = ObfuscatedID::new(&from).map_err(Error::new_with_source)?;

            let selected_type = selected_value
                .await
                .map_err(Error::new_with_source)?
                .into_iter()
                .map(|((selected_pk, _), _)| {
                    let to_ty = ObfuscatedID::new(&selected_pk).map_err(Error::new_with_source)?;
                    let transaction = PossibleChanges::new_link_cached(
                        to_ty.ty().to_string(),
                        to_ty.id().to_string(),
                        from_ty.ty().to_string(),
                        from_ty.id().to_string(),
                        relation.name.clone(),
                        parent_value.clone(),
                    );
                    Ok(transaction)
                })
                .collect::<Result<Vec<PossibleChanges>, Error>>()?;

            transaction_batcher
                .load_many(selected_type)
                .await
                .map_err(Error::new_with_source)?;

            Ok(ResolvedValue::new(Arc::new(serde_json::Value::Null)))
        }
        _ => Ok(ResolvedValue::new(Arc::new(serde_json::Value::Null))),
    }
}

fn internal_node_linking<'a>(
    ctx: &'a Context<'a>,
    parent_ty: &'a MetaType,
    child_ty: &'a MetaType,
    parent_value: SharedSelectionType<'a>,
    relation_name: &'a str,
    linking_input: &Value,
) -> RecursiveCreation<'a> {
    // For linking, it's either, Id, Array of Id, or Null
    let field_value = match linking_input {
        Value::String(inner) => Some(vec![(inner.clone(), inner.clone())]),
        Value::List(list) => Some(
            list.iter()
                .map(|value| match value {
                    Value::String(inner) => (inner.clone(), inner.clone()),
                    _ => panic!(),
                })
                .collect(),
        ),
        _ => None,
    };

    let selection: SelectionType<'a> = match field_value {
        Some(field_value) => Box::pin(async move {
            let batchers = ctx.data_unchecked::<Arc<DynamoDBBatchersData>>();
            let loader_batcher = &batchers.loader;

            loader_batcher.load_many(field_value).await
        }),
        None => Box::pin(async move { Ok(HashMap::new()) }),
    };
    let shared_selection = selection.shared();

    let create_normal_future: TransactionType<'a> = Box::pin(create_relation_node(
        ctx,
        parent_ty,
        shared_selection.clone(),
        parent_value.clone(),
        relation_name,
    ));

    let create_reverse_future: TransactionType<'a> = Box::pin(create_relation_node(
        ctx,
        child_ty,
        parent_value,
        shared_selection.clone(),
        relation_name,
    ));

    RecursiveCreation {
        selection: shared_selection,
        transaction: vec![create_normal_future, create_reverse_future],
    }
}

fn internal_node_unlinking<'a>(
    ctx: &'a Context<'a>,
    parent_value: SharedSelectionType<'a>,
    relation_name: &'a str,
    unlinking_input: &Value,
) -> RecursiveCreation<'a> {
    // For unlinking, it's either, Id, Array of Id, or Null
    let field_value = match unlinking_input {
        Value::String(inner) => Some(vec![(inner.clone(), inner.clone())]),
        Value::List(list) => Some(
            list.iter()
                .map(|value| match value {
                    Value::String(inner) => (inner.clone(), inner.clone()),
                    _ => panic!(),
                })
                .collect(),
        ),
        _ => None,
    };

    if let Some(field_value) = field_value {
        let field_value_clone = field_value.clone();
        let selection: SelectionType<'a> = Box::pin(async move {
            let batchers = ctx.data_unchecked::<Arc<DynamoDBBatchersData>>();
            let loader_batcher = &batchers.loader;

            loader_batcher.load_many(field_value_clone).await
        });

        let shared_selection = selection.shared();

        let mut transactions = Vec::with_capacity(field_value.len() + 1);

        for (pk, _) in field_value {
            let a: TransactionType<'a> = Box::pin(relation_remove(
                ctx,
                parent_value.clone(),
                pk,
                relation_name,
            ));
            transactions.push(a);
        }

        RecursiveCreation {
            selection: shared_selection,
            transaction: transactions,
        }
    } else {
        ctx.add_error(ServerError::new(
            "If you fill an unlink value it shouldn't be null",
            Some(ctx.item.pos),
        ));
        let selection: SelectionType<'a> = Box::pin(async move { Ok(HashMap::new()) });

        RecursiveCreation {
            selection: selection.shared(),
            transaction: Vec::new(),
        }
    }
}

/// This function will be used into creation / udpate of a relation:
///
/// When we create a new Node, we'll have relations over this node, those relations,
/// depending on the input, will need to create:
///     - The sub-node if it's a Create
///     - The relations between the parent-node and the sub-node
///     - The relations between the sub-node and the parent-node
///
/// When we update a new Node, for a relation it means it can be:
///     - Same as in the creation flow
///     - unlink a Relation which means we'll need to delete the both side of the relation.
///
/// This function will return the Futures which will be used to create those relations
/// and also the Futures which will be used to have the Projected Data
///
/// Recursive function which create a future to either fetch or create an item.
/// So we need to:
/// - Get the sub-input of a type
/// - If we need to link it, we'll only need to fetch it to get the value.
/// - If we do need to create it, and there are sub-types,
///     we'll also need to create the sub-types of this type and the relation
/// - If we do need to create it, and there aren't any sub-types we can trigger
///     the atomic creation.
///
/// # Optimization concern
/// It should be recursive, but not be runned recursively, we want to have every
/// fetch and write optimized into the less affordable number of queries.
///
#[allow(clippy::too_many_arguments)]
fn relation_handle<'a>(
    ctx: &'a Context<'a>,
    parent_ty: &'a MetaType,
    parent_value: SharedSelectionType<'a>,
    relation_field: &'a str,
    relation_name: &'a str,
    input: &IndexMap<Name, Value>,
    execution_id: Ulid,
    increment: Arc<AtomicUsize>,
) -> Vec<RecursiveCreation<'a>> {
    let child_ty_name = &type_to_base_type(
        &parent_ty
            .field_by_name(relation_field)
            .unwrap()
            .ty
            // we want to persist the underlying type rather than the connection
            // TODO: look into other methods of doing this
            .trim_end_matches("Connection"),
    )
    .unwrap();

    // We determinate the subtype of this relation
    let child_ty: &MetaType = ctx.registry().types.get(child_ty_name).unwrap();

    // We need to tell if it's a `create` or a `link`
    // So we get the child input first
    let child_input = match input.get(&Name::new(relation_field)).and_then(inputs) {
        Some(val) => val,
        _ => {
            return Vec::new();
        }
    };

    let mut result = Vec::with_capacity(child_input.len());

    for child_input in child_input {
        let create = child_input.get("create");
        let link = child_input.get("link");
        let unlink = child_input.get("unlink");
        let parent_value = parent_value.clone();

        let result_local = match (create, link, unlink) {
            (Some(Value::Object(creation_input)), None, None) => {
                let mut result = node_create(
                    ctx,
                    child_ty,
                    execution_id,
                    increment.clone(),
                    creation_input.clone(),
                );

                let shared_selection_cloned = result.selection.clone();

                let create_normal_future: TransactionType<'a> = Box::pin(create_relation_node(
                    ctx,
                    parent_ty,
                    shared_selection_cloned.clone(),
                    parent_value.clone(),
                    relation_name,
                ));

                let create_reverse_future: TransactionType<'a> = Box::pin(create_relation_node(
                    ctx,
                    child_ty,
                    parent_value,
                    shared_selection_cloned,
                    relation_name,
                ));

                result
                    .transaction
                    .extend(vec![create_normal_future, create_reverse_future]);

                result
            }
            (None, Some(linking_input), None) => internal_node_linking(
                ctx,
                parent_ty,
                child_ty,
                parent_value.clone(),
                relation_name,
                linking_input,
            ),
            (None, None, Some(unlinking_input)) => {
                internal_node_unlinking(ctx, parent_value.clone(), relation_name, unlinking_input)
            }
            (None, Some(linking_input), Some(unlinking_input)) => {
                internal_node_unlinking(ctx, parent_value.clone(), relation_name, unlinking_input)
                    + internal_node_linking(
                        ctx,
                        parent_ty,
                        child_ty,
                        parent_value.clone(),
                        relation_name,
                        linking_input,
                    )
            }
            _ => {
                let selection: SelectionType<'a> = Box::pin(async move { Ok(HashMap::new()) });
                RecursiveCreation {
                    selection: selection.shared(),
                    transaction: Vec::new(),
                }
            }
        };

        result.push(result_local);
    }

    result
}

#[async_trait::async_trait]
impl ResolverTrait for DynamoMutationResolver {
    async fn resolve(
        &self,
        ctx: &Context<'_>,
        resolver_ctx: &ResolverContext<'_>,
        last_resolver_value: Option<&ResolvedValue>,
    ) -> Result<ResolvedValue, Error> {
        let batchers = &ctx.data::<Arc<DynamoDBBatchersData>>()?;

        match self {
            // This one is tricky, when we create a new node, we have to check that the node do not
            // contains any Edges on the first level. If there is an edge at the first level we
            // need to fetch this edge as a node and store it alongside the actual node.
            //
            // Why?
            //
            // Because it's how we store the data.
            DynamoMutationResolver::CreateNode { input, ty } => {
                let ctx_ty = ctx.registry().types.get(ty).ok_or_else(|| {
                    Error::new("Internal Error: Failed process the associated schema.")
                })?;

                let ctx_create_input_ty = ctx
                    .registry()
                    .types
                    .get(&format!("{ty}CreateInput"))
                    .ok_or_else(|| {
                        Error::new("Internal Error: Failed process the associated schema.")
                    })?;

                let create_input_fields = match ctx_create_input_ty {
                    MetaType::InputObject { input_fields, .. } => input_fields,
                    _ => {
                        return Err(Error::new(
                            "Internal Error: `*CreateInput` type is not an input object",
                        ))
                    }
                };

                let id = resolver_ctx.execution_id.to_string();
                let autogenerated_id = NodeID::new(&ty, &id);

                let input = match input
                    .param(ctx, last_resolver_value.map(|x| x.data_resolved.borrow()))?
                    .expect("can't fail")
                {
                    Value::Object(inner) => inner,
                    _ => {
                        return Err(Error::new("Internal Error: failed to infer key"));
                    }
                };

                // Extend with default values for the fields missing in the input.
                // Values from `input` take precedence.
                let input = create_input_fields
                    .iter()
                    .filter_map(|(name, field)| {
                        field
                            .default_value
                            .as_ref()
                            .map(|default_value| (Name::new(name.as_str()), default_value.clone()))
                    })
                    .chain(input)
                    .collect();

                let creation = node_create(
                    ctx,
                    ctx_ty,
                    *resolver_ctx.execution_id,
                    Arc::new(AtomicUsize::new(0)),
                    input,
                );
                let _ = creation.selection.await?;
                let _ = futures_util::future::try_join_all(creation.transaction).await?;

                Ok(ResolvedValue::new(Arc::new(serde_json::json!({
                    "id": serde_json::Value::String(autogenerated_id.to_string()),
                }))))
            }
            DynamoMutationResolver::UpdateNode { id, input, ty } => {
                let ctx_ty = ctx.registry().types.get(ty).ok_or_else(|| {
                    Error::new("Internal Error: Failed process the associated schema.")
                })?;

                let id =
                    id.expect_string(ctx, last_resolver_value.map(|x| x.data_resolved.borrow()))?;

                ObfuscatedID::expect(&id, &ty)
                    .map_err(|err| err.into_server_error(ctx.item.pos))?;

                let input =
                    input.expect_obj(ctx, last_resolver_value.map(|x| x.data_resolved.borrow()))?;

                let update = node_update(
                    ctx,
                    ctx_ty,
                    *resolver_ctx.execution_id,
                    Arc::new(AtomicUsize::new(0)),
                    input,
                    id.clone(),
                );

                let _ = update.selection.await?;

                let mut stream = futures_util::stream::iter(update.transaction.into_iter())
                    .buffer_unordered(100);

                while let Some(result) = stream.next().await {
                    result?;
                }

                Ok(ResolvedValue::new(Arc::new(serde_json::json!({
                    "id": serde_json::Value::String(id),
                }))))
            }
            DynamoMutationResolver::DeleteNode { id, ty } => {
                let new_transaction = &batchers.transaction_new;
                let id_to_be_deleted =
                    id.expect_string(ctx, last_resolver_value.map(|x| x.data_resolved.borrow()))?;

                let opaque_id = ObfuscatedID::expect(&id_to_be_deleted, &ty)
                    .map_err(|err| err.into_server_error(ctx.item.pos))?;

                let ty = opaque_id.ty().to_string();
                let id = opaque_id.id().to_string();

                new_transaction
                    .load_one(PossibleChanges::delete_node(ty.clone(), id.clone()))
                    .await?;

                Ok(ResolvedValue::new(Arc::new(serde_json::json!({
                    "id": serde_json::Value::String(id_to_be_deleted),
                }))))
            }
        }
    }
}

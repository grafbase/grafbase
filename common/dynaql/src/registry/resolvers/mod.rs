//! Resolvers dynamic strategy explained here.
//!
//! A Resolver is a part of the way we resolve a field. It's an asynchronous
//! operation which is cached based on his id and on his execution_id.
//!
//! When you `resolve` a Resolver, you have access to the `ResolverContext`
//! which will grant you access to the current `Transformer` that must be
//! applied to this resolve, after getting the data by the resolvers.
//!
//! A Resolver always know how to apply the associated transformers.

use std::borrow::Cow;

use crate::{dynamic::DynamicFieldContext, ServerError, ServerResult};

use context_data::ParentDataResolver;

use dynamo_mutation::MutationResolver;
use dynamo_querying::QueryResolver;
use dynamodb::PaginatedCursor;

use std::hash::Hash;

use graph_entities::{cursor::PaginationCursor, NodeID};
use serde::{de::DeserializeOwned, Serialize};

use super::MetaType;

pub mod context_data;
pub mod dynamo_mutation;
pub mod dynamo_querying;

pub const PAGINATION_HAS_NEXT_PAGE: &str = "has_next_page";
pub const PAGINATION_HAS_PREVIOUS_PAGE: &str = "has_previous_page";
pub const PAGINATION_START_CURSOR: &str = "start_cursor";
pub const PAGINATION_END_CURSOR: &str = "end_cursor";

#[derive(Debug, Hash, Clone)]
pub enum ResolvedPaginationDirection {
    Forward,
    Backward,
}

impl ResolvedPaginationDirection {
    pub fn from_paginated_cursor(cursor: &PaginatedCursor) -> Self {
        match cursor {
            PaginatedCursor::Forward { .. } => Self::Forward,
            PaginatedCursor::Backward { .. } => Self::Backward,
        }
    }
}

#[derive(Debug, Hash, Clone)]
pub struct ResolvedPaginationInfo {
    pub direction: ResolvedPaginationDirection,
    pub end_cursor: Option<PaginationCursor>,
    pub start_cursor: Option<PaginationCursor>,
    pub more_data: bool,
}

impl ResolvedPaginationInfo {
    pub fn new(direction: ResolvedPaginationDirection) -> Self {
        Self {
            direction,
            end_cursor: None,
            start_cursor: None,
            more_data: false,
        }
    }

    pub fn with_start(mut self, start_cursor: Option<PaginationCursor>) -> Self {
        self.start_cursor = start_cursor;
        self
    }

    pub fn with_end(mut self, end_cursor: Option<PaginationCursor>) -> Self {
        self.end_cursor = end_cursor;
        self
    }

    pub fn with_more_data(mut self, data: bool) -> Self {
        self.more_data = data;
        self
    }

    pub fn output(&self) -> serde_json::Value {
        let has_next_page = matches!(
            (&self.direction, self.more_data),
            (&ResolvedPaginationDirection::Forward, true)
        );

        let has_previous_page = matches!(
            (&self.direction, self.more_data),
            (&ResolvedPaginationDirection::Backward, true)
        );

        serde_json::json!({
            PAGINATION_HAS_NEXT_PAGE: has_next_page,
            PAGINATION_HAS_PREVIOUS_PAGE: has_previous_page,
            PAGINATION_START_CURSOR: self.start_cursor,
            PAGINATION_END_CURSOR: self.end_cursor,
        })
    }
}

#[derive(Debug)]
pub struct ResolvedContainer<'a> {
    pub maybe_node_id: Option<NodeID<'a>>,
    pub value: ResolvedValue<'a>,
}

impl<'a> ResolvedContainer<'a> {
    pub fn new(base_type: &MetaType, value: ResolvedValue<'a>) -> Self {
        ResolvedContainer {
            // FIXME: It relies implicitly on DynamoDB format, see node_id() and is executed even
            // for containers for which an doesn't exist yet like '<x>Collection' fields.
            maybe_node_id: value
                .node_id(base_type.name())
                .and_then(|id| NodeID::from_owned(id).ok()),
            value,
        }
    }
}

/// ResolvedValue are values passed arround between resolvers, it contains the actual Resolver data
/// but will also contain other informations wich may be use later by custom resolvers, like for
/// example Pagination Details.
///
/// Cheap to Clone
#[derive(Debug, Clone)]
pub struct ResolvedValue<'a> {
    /// Data Resolved by the current Resolver
    pub value: Cow<'a, serde_json::Value>,
    /// Optional pagination data for Paginated Resolvers
    pub pagination: Option<ResolvedPaginationInfo>,
}

impl<'a> ResolvedValue<'a> {
    pub fn null() -> Self {
        Self::owned(serde_json::Value::Null)
    }

    pub fn new(value: Cow<'a, serde_json::Value>) -> Self {
        ResolvedValue {
            value,
            pagination: None,
        }
    }

    pub fn borrowed(value: &'a serde_json::Value) -> Self {
        ResolvedValue {
            value: Cow::Borrowed(value),
            pagination: None,
        }
    }

    pub fn owned(value: serde_json::Value) -> Self {
        ResolvedValue {
            value: Cow::Owned(value),
            pagination: None,
        }
    }

    pub fn with_pagination(mut self, pagination: ResolvedPaginationInfo) -> Self {
        self.pagination = Some(pagination);
        self
    }

    /// FIXME: This currently relies on the internal structure of DynamoDB response
    ///
    /// We can check from the schema definition if it's a node, if it is, we need to
    /// have a way to get it
    /// temp: Little hack here, we know that `ResolvedValue` are bound to have a format
    /// of:
    /// ```ignore
    /// {
    ///   "Node": {
    ///     "__sk": {
    ///       "S": "node_id"
    ///     }
    ///   }
    /// }
    /// ```
    /// We use that fact without checking it here.
    ///
    /// This have to be removed when we rework registry & dynaql to have a proper query
    /// planning.
    pub fn node_id<S: AsRef<str>>(&self, entity: S) -> Option<String> {
        self.value.get(entity.as_ref()).and_then(|x| {
            x.get("__sk")
                .and_then(|x| {
                    if let serde_json::Value::Object(value) = x {
                        Some(value)
                    } else {
                        None
                    }
                })
                .and_then(|x| x.get("S"))
                .and_then(|value| {
                    if let serde_json::Value::String(value) = value {
                        Some(value.clone())
                    } else {
                        None
                    }
                })
        })
    }
}

impl Resolver {
    pub fn field(key: &str) -> Self {
        Self::parent(ParentDataResolver::Field(key.to_string()))
    }

    pub fn parent_object() -> Self {
        Self::parent(ParentDataResolver::Clone)
    }

    pub fn dynamo_attr(key: &str) -> Self {
        Self::parent(ParentDataResolver::DynamoAttribute(key.to_string()))
    }

    pub fn constant<T: Serialize>(value: T) -> Self {
        Self::Constant(HashableJsonValue(
            serde_json::to_value(value).expect("Constant arguments must be serializable"),
        ))
    }

    pub fn input(name: &str) -> Self {
        Self::InputValue(name.to_string())
    }

    pub fn parent(r: ParentDataResolver) -> Self {
        Self::ParentData(Box::new(r))
    }

    pub fn query(r: QueryResolver) -> Self {
        Self::Query(Box::new(r))
    }

    pub fn mutation(r: MutationResolver) -> Self {
        Self::Mutation(Box::new(r))
    }

    pub async fn resolve_oneof<'ctx>(
        &self,
        ctx_field: &DynamicFieldContext<'ctx>,
        maybe_parent_value: Option<&ResolvedValue<'ctx>>,
    ) -> ServerResult<(String, serde_json::Value)> {
        self.resolve_object(ctx_field, maybe_parent_value)
            .await?
            .into_iter()
            // We don't need to check whether multiple fields are specified or not.
            .next()
            .map(Ok)
            .unwrap_or_else(|| {
                // Shouldn't happen, resolver_input() should already have raised an error at this point.
                // FIXME: Add validation test and replace this part with unreachable!()
                Err(ServerError::new(
                    "Invalid empty @oneof object",
                    Some(ctx_field.item.pos),
                ))
            })
    }

    pub async fn resolve_object<'ctx>(
        &self,
        ctx_field: &DynamicFieldContext<'ctx>,
        maybe_parent_value: Option<&ResolvedValue<'ctx>>,
    ) -> ServerResult<serde_json::Map<String, serde_json::Value>> {
        match self
            .resolve_dynamic(ctx_field, maybe_parent_value)
            .await?
            .value
            .into_owned()
        {
            serde_json::Value::Object(m) => Ok(m),
            _ => Err(ServerError::new(
                "Expected an object",
                Some(ctx_field.item.pos),
            )),
        }
    }

    pub async fn resolve<T: DeserializeOwned>(
        &self,
        ctx_field: &DynamicFieldContext<'_>,
        maybe_parent_value: Option<&ResolvedValue<'_>>,
    ) -> ServerResult<T> {
        let resolve = self.resolve_dynamic(ctx_field, maybe_parent_value).await?;
        serde_json::from_value(resolve.value.into_owned())
            .map_err(|err| ServerError::new(err.to_string(), Some(ctx_field.item.pos)))
    }

    #[async_recursion::async_recursion]
    pub async fn resolve_dynamic<'ctx>(
        &self,
        ctx_field: &DynamicFieldContext<'ctx>,
        maybe_parent_value: Option<&'async_recursion ResolvedValue<'ctx>>,
    ) -> ServerResult<ResolvedValue<'ctx>> {
        match self {
            Self::Query(dynamodb) => dynamodb
                .resolve(ctx_field, maybe_parent_value)
                .await
                .map_err(|err| err.into_server_error(ctx_field.item.pos)),
            Self::Mutation(dynamodb) => dynamodb
                .resolve(ctx_field, maybe_parent_value)
                .await
                .map_err(|err| err.into_server_error(ctx_field.item.pos)),
            Self::ParentData(parent) => parent
                .resolve(ctx_field, maybe_parent_value)
                .await
                .map_err(|err| err.into_server_error(ctx_field.item.pos)),
            Self::Constant(value) => Ok(ResolvedValue::owned(value.0.clone())),
            Self::InputValue(name) => ctx_field
                .param_value_dynamic(name)
                .map(ResolvedValue::owned),
            Self::Composition(resolvers) => {
                let [head, tail @ ..] = &resolvers[..] else {
                    unreachable!("Composition of resolvers always have at least one element")
                };
                let mut current = head.resolve_dynamic(ctx_field, maybe_parent_value).await?;
                for resolver in tail {
                    current = resolver.resolve_dynamic(ctx_field, Some(&current)).await?;
                }
                Ok(current)
            }
        }
    }

    pub fn and_then(self, resolver: Resolver) -> Self {
        let mut resolvers = match self {
            Self::Composition(resolvers) => resolvers,
            _ => vec![self],
        };
        resolvers.push(resolver);
        Self::Composition(resolvers)
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum Resolver {
    // Boxing the actual resolvers to allow them to rely on other Resolvers.
    Query(Box<QueryResolver>),
    Mutation(Box<MutationResolver>),
    ParentData(Box<ParentDataResolver>),
    Constant(HashableJsonValue),
    InputValue(String),
    Composition(Vec<Resolver>),
}

// A bit hacky, not sure if there's a better approach...
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Eq)]
pub struct HashableJsonValue(serde_json::Value);

impl Hash for HashableJsonValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        serde_json_hash(&self.0, state)
    }
}

impl PartialEq for HashableJsonValue {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

fn serde_json_hash<H: std::hash::Hasher>(value: &serde_json::Value, state: &mut H) {
    match value {
        serde_json::Value::Null => false.hash(state),
        serde_json::Value::Bool(b) => b.hash(state),
        serde_json::Value::Number(n) => n.hash(state),
        serde_json::Value::String(s) => s.hash(state),
        serde_json::Value::Array(a) => {
            for v in a {
                serde_json_hash(v, state);
            }
        }
        serde_json::Value::Object(m) => {
            for (k, v) in m {
                k.hash(state);
                serde_json_hash(v, state);
            }
        }
    }
}

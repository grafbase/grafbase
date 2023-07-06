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

use self::{custom::CustomResolver, graphql::Target, transformer::Transformer};
use crate::{Context, Error, RequestHeaders};
use derivative::Derivative;
use dynamo_mutation::DynamoMutationResolver;
use dynamo_querying::DynamoResolver;
use dynamodb::PaginatedCursor;
use dynaql_parser::types::SelectionSet;
use dynaql_value::{ConstValue, Name};
use grafbase_runtime::cursor::Cursor;
use graph_entities::ConstraintID;
use query::QueryResolver;

use std::{borrow::Borrow, sync::Arc};
use ulid::Ulid;

use super::{Constraint, MetaField, MetaType};

pub mod atlas_data_api;
pub mod custom;
pub mod dynamo_mutation;
pub mod dynamo_querying;
pub mod graphql;
pub mod http;
pub mod query;
pub mod transformer;

/// Resolver Context
///
/// Each time a Resolver is accessed to be resolved, a context for the resolving
/// strategy is created.
///
/// This context contain safe access data to be used inside `ResolverTrait`.
/// This give you access to the `resolver_id` which define the resolver, the
/// `execution_id` which is linked to the actual execution, a unique ID is
/// created each time the resolver is called.
#[derive(Debug)]
pub struct ResolverContext<'a> {
    /// When a resolver is executed, it gains a Resolver unique ID for his
    /// execution, this ID is used for internal cache strategy
    pub execution_id: &'a Ulid,
    /// The current Type being resolved if we know it. It's the type linked to the resolver.
    pub ty: Option<&'a MetaType>,
    /// The current SelectionSet.
    pub selections: Option<&'a SelectionSet>,
    /// The current field being resolved if we know it.
    pub field: Option<&'a MetaField>,
}

impl<'a> ResolverContext<'a> {
    pub fn new(id: &'a Ulid) -> Self {
        Self {
            execution_id: id,
            ty: None,
            selections: None,
            field: None,
        }
    }

    pub fn with_ty(mut self, ty: Option<&'a MetaType>) -> Self {
        self.ty = ty;
        self
    }

    pub fn with_field(mut self, field: Option<&'a MetaField>) -> Self {
        self.field = field;
        self
    }

    pub fn with_selection_set(mut self, selections: Option<&'a SelectionSet>) -> Self {
        self.selections = selections;
        self
    }
}

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
    pub start_cursor: Option<Cursor>,
    pub end_cursor: Option<Cursor>,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

impl ResolvedPaginationInfo {
    pub fn of<C: Into<Cursor>>(
        direction: ResolvedPaginationDirection,
        start_cursor: Option<C>,
        end_cursor: Option<C>,
        more_data: bool,
    ) -> Self {
        Self {
            start_cursor: start_cursor.map(Into::into),
            end_cursor: end_cursor.map(Into::into),
            has_next_page: matches!(
                (&direction, more_data),
                (&ResolvedPaginationDirection::Forward, true)
            ),
            has_previous_page: matches!(
                (&direction, more_data),
                (&ResolvedPaginationDirection::Backward, true)
            ),
        }
    }

    pub fn output(&self) -> serde_json::Value {
        serde_json::json!({
            "has_next_page": self.has_next_page,
            "has_previous_page": self.has_previous_page,
            "start_cursor": self.start_cursor,
            "end_cursor": self.end_cursor,
        })
    }
}

/// ResolvedValue are values passed arround between resolvers, it contains the actual Resolver data
/// but will also contain other informations wich may be use later by custom resolvers, like for
/// example Pagination Details.
///
/// Cheap to Clone
#[derive(Debug, Derivative, Clone)]
#[derivative(Hash)]
pub struct ResolvedValue {
    /// Data Resolved by the current Resolver.
    ///
    /// The data is sent as-is to the next resolver in the chain. The format of the data is
    /// dependent on the resolver that produced the data.
    ///
    /// For example, the GraphQL resolver returns data in the actual shape of the query. That is, a
    /// resolver that resolves a `user { name }` query, is expected to return a `{ "user": { "name"
    /// "..." } }` JSON object.
    ///
    /// Other resolvers might transform/augment the data before passing it along.
    #[derivative(Hash = "ignore")]
    pub data_resolved: Arc<serde_json::Value>,
    /// Optional pagination data for Paginated Resolvers
    pub pagination: Option<ResolvedPaginationInfo>,
    /// Resolvers can set this value when resolving so the engine will know it's
    /// not usefull to continue iterating over the ResolverChain.
    pub early_return_null: bool,
}

impl Borrow<serde_json::Value> for &ResolvedValue {
    fn borrow(&self) -> &serde_json::Value {
        self.data_resolved.borrow()
    }
}

impl ResolvedValue {
    pub fn new(value: Arc<serde_json::Value>) -> Self {
        Self {
            data_resolved: value,
            pagination: None,
            early_return_null: false,
        }
    }

    pub fn null() -> Self {
        Self::new(Arc::new(serde_json::Value::Null))
    }

    pub fn with_pagination(mut self, pagination: ResolvedPaginationInfo) -> Self {
        self.pagination = Some(pagination);
        self
    }

    pub fn with_early_return(mut self) -> Self {
        self.early_return_null = true;
        self
    }

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
        self.data_resolved.get(entity.as_ref()).and_then(|x| {
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

    pub fn is_early_returned(&self) -> bool {
        self.early_return_null
    }
}

impl Resolver {
    #[async_recursion::async_recursion]
    pub(crate) async fn resolve(
        &self,
        ctx: &Context<'_>,
        resolver_ctx: &ResolverContext<'_>,
        last_resolver_value: Option<&'async_recursion ResolvedValue>,
    ) -> Result<ResolvedValue, Error> {
        match self {
            Resolver::Parent => last_resolver_value
                .cloned()
                .ok_or_else(|| Error::new("No data to propagate!")),
            Resolver::DynamoResolver(dynamodb) => {
                dynamodb
                    .resolve(ctx, resolver_ctx, last_resolver_value)
                    .await
            }
            Resolver::DynamoMutationResolver(dynamodb) => {
                dynamodb
                    .resolve(ctx, resolver_ctx, last_resolver_value)
                    .await
            }
            Resolver::Transformer(ctx_data) => {
                ctx_data
                    .resolve(ctx, resolver_ctx, last_resolver_value)
                    .await
            }
            Resolver::CustomResolver(resolver) => resolver.resolve(ctx, last_resolver_value).await,
            Resolver::Query(query) => query.resolve(ctx, resolver_ctx, last_resolver_value).await,
            Resolver::Composition(resolvers) => {
                let [head, tail @ ..] = &resolvers[..] else {
                    unreachable!("Composition of resolvers always have at least one element")
                };
                let mut current = head.resolve(ctx, resolver_ctx, last_resolver_value).await?;
                for resolver in tail {
                    current = resolver.resolve(ctx, resolver_ctx, Some(&current)).await?;
                }
                Ok(current)
            }
            Resolver::Http(resolver) => {
                resolver
                    .resolve(ctx, resolver_ctx, last_resolver_value)
                    .await
            }
            Resolver::Graphql(resolver) => {
                let registry = ctx.registry();
                let request_headers = ctx.data::<RequestHeaders>().ok();
                let headers = registry
                    .http_headers
                    .get(&format!("GraphQLConnector{}", resolver.id))
                    .zip(request_headers)
                    .map(|(connector_headers, request_headers)| {
                        connector_headers.build_header_vec(request_headers)
                    })
                    .unwrap_or_default();

                let fragment_definitions = ctx
                    .query_env
                    .fragments
                    .iter()
                    .map(|(k, v)| (k, v.as_ref().node))
                    .collect();

                let variable_definitions = ctx
                    .query_env
                    .operation
                    .node
                    .variable_definitions
                    .iter()
                    .map(|variable_definition| {
                        (
                            &variable_definition.node.name.node,
                            &variable_definition.node,
                        )
                    })
                    .collect();

                let target = match resolver.namespace {
                    Some(_) => {
                        let current_object = resolver_ctx
                            .ty
                            .ok_or_else(|| Error::new("Internal error"))?
                            .try_into()
                            .map_err(|_| Error::new("Internal error"))?;

                        Target::SelectionSet(
                            Box::new(
                                ctx.item
                                    .node
                                    .selection_set
                                    .node
                                    .items
                                    .as_slice()
                                    .iter()
                                    .map(|v| &v.node),
                            ),
                            current_object,
                        )
                    }
                    None => Target::Field(
                        &ctx.item,
                        resolver_ctx
                            .field
                            .ok_or_else(|| Error::new("internal error"))?,
                    ),
                };

                let operation = ctx.query_env.operation.node.ty;
                let error_handler = |error| ctx.add_error(error);
                let variables = ctx.query_env.variables.clone();

                resolver
                    .resolve(
                        // Be a lot easier to just pass the context in here...
                        operation,
                        &headers,
                        fragment_definitions,
                        target,
                        error_handler,
                        variables,
                        variable_definitions,
                        registry,
                    )
                    .await
                    .map_err(Into::into)
            }
            Resolver::MongoResolver(resolver) => {
                Ok(resolver.resolve(ctx, resolver_ctx).await.unwrap())
            }
        }
    }

    pub fn and_then(mut self, resolver: impl Into<Resolver>) -> Self {
        let resolver = resolver.into();
        match &mut self {
            Resolver::Composition(resolvers) => {
                resolvers.push(resolver);
                self
            }
            _ => Resolver::Composition(vec![self, resolver]),
        }
    }

    pub fn and_then_maybe(self, resolver: Option<impl Into<Resolver>>) -> Self {
        match resolver {
            Some(other) => self.and_then(other),
            None => self,
        }
    }

    pub fn is_parent(&self) -> bool {
        matches!(self, Self::Parent)
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Self::CustomResolver(_))
    }
}

#[non_exhaustive]
#[serde_with::minify_variant_names(serialize = "minified", deserialize = "minified")]
#[derive(Default, Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum Resolver {
    // By default a resolver will just return its parent value
    #[default]
    Parent,
    DynamoResolver(DynamoResolver),
    Query(QueryResolver),
    DynamoMutationResolver(DynamoMutationResolver),
    Transformer(Transformer),
    CustomResolver(CustomResolver),
    Composition(Vec<Resolver>),
    Http(http::HttpResolver),
    Graphql(graphql::Resolver),
    MongoResolver(atlas_data_api::AtlasDataApiResolver),
}

impl Constraint {
    /// Extracts a ConstraintID for this constraint from the corresponding field
    /// of a `*ByInput` type.
    ///
    /// If the constraint has one field we expect the value to just be a string.
    /// If the constraint has multiple it should be an Object of fieldName: value
    pub fn extract_id_from_by_input_field(
        &self,
        ty: &str,
        value: &ConstValue,
    ) -> Option<ConstraintID<'static>> {
        let fields = self.fields();
        if fields.len() == 1 {
            return Some(ConstraintID::new(
                ty.to_string(),
                vec![(fields[0].clone(), value.clone().into_json().ok()?)],
            ));
        }

        let ConstValue::Object(by_fields) = value else {
            return None;
        };

        let constraint_fields = fields
            .into_iter()
            .map(|field| {
                Some((
                    field.clone(),
                    by_fields.get(&Name::new(field))?.clone().into_json().ok()?,
                ))
            })
            .collect::<Option<Vec<_>>>()?;

        Some(ConstraintID::new(ty.to_string(), constraint_fields))
    }
}

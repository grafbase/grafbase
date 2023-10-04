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

use dynamo_mutation::DynamoMutationResolver;
use dynamo_querying::DynamoResolver;
use dynamodb::PaginatedCursor;
use engine_parser::types::SelectionSet;
use engine_value::{ConstValue, Name};
use graph_entities::ConstraintID;
use query::QueryResolver;
use runtime::search::GraphqlCursor;
use ulid::Ulid;

pub use self::resolved_value::ResolvedValue;
use self::{
    custom::CustomResolver,
    federation::resolve_federation_entities,
    graphql::{QueryBatcher, Target},
    transformer::Transformer,
};
use super::{type_kinds::OutputType, Constraint, MetaField};
use crate::{Context, ContextExt, ContextField, Error, RequestHeaders};

pub mod atlas_data_api;
pub mod custom;
pub mod dynamo_mutation;
pub mod dynamo_querying;
mod federation;
pub mod graphql;
pub mod http;
mod logged_fetch;
pub mod postgresql;
pub mod query;
mod resolved_value;
pub mod transformer;

use tracing::{info_span, Instrument};

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
    /// The current Type being resolved. It's the type linked to the resolver.
    pub ty: OutputType<'a>,
    /// The current SelectionSet.
    pub selections: &'a SelectionSet,
    /// The current field being resolved
    pub field: &'a MetaField,
}

impl<'a> ResolverContext<'a> {
    pub fn new(field_context: &'a ContextField<'a>) -> Self {
        Self {
            execution_id: &field_context.execution_id,
            ty: field_context.field_base_type(),
            selections: &field_context.item.selection_set.node,
            field: field_context.field,
        }
    }

    /// Can be used to update the type in this ResolverContext.
    ///
    /// Useful in cases where the original ty is an interface/union and we
    /// know specifically which underlying type we're working with
    pub fn with_ty(self, ty: OutputType<'a>) -> Self {
        Self { ty, ..self }
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
    pub start_cursor: Option<GraphqlCursor>,
    pub end_cursor: Option<GraphqlCursor>,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

impl ResolvedPaginationInfo {
    pub fn of<C: Into<GraphqlCursor>>(
        direction: ResolvedPaginationDirection,
        start_cursor: Option<C>,
        end_cursor: Option<C>,
        more_data: bool,
    ) -> Self {
        Self {
            start_cursor: start_cursor.map(Into::into),
            end_cursor: end_cursor.map(Into::into),
            has_next_page: matches!((&direction, more_data), (&ResolvedPaginationDirection::Forward, true)),
            has_previous_page: matches!((&direction, more_data), (&ResolvedPaginationDirection::Backward, true)),
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

impl Resolver {
    #[async_recursion::async_recursion]
    pub(crate) async fn resolve(
        &self,
        ctx: &ContextField<'_>,
        resolver_ctx: &ResolverContext<'_>,
        last_resolver_value: Option<ResolvedValue>,
    ) -> Result<ResolvedValue, Error> {
        match self {
            Resolver::Parent => last_resolver_value.ok_or_else(|| Error::new("No data to propagate!")),
            Resolver::DynamoResolver(dynamodb) => {
                dynamodb
                    .resolve(ctx, resolver_ctx, last_resolver_value.as_ref())
                    .instrument(info_span!("dynamo_resolver"))
                    .await
            }
            Resolver::DynamoMutationResolver(dynamodb) => {
                dynamodb
                    .resolve(ctx, resolver_ctx, last_resolver_value.as_ref())
                    .instrument(info_span!("dynamo_mutation_resolver"))
                    .await
            }
            Resolver::Transformer(ctx_data) => ctx_data.resolve(ctx, resolver_ctx, last_resolver_value).await,
            Resolver::CustomResolver(resolver) => {
                resolver
                    .resolve(ctx, last_resolver_value.as_ref())
                    .instrument(info_span!("custom_resolver", resolver_name = resolver.resolver_name))
                    .await
            }
            Resolver::Query(query) => query.resolve(ctx, resolver_ctx, last_resolver_value.as_ref()).await,
            Resolver::Composition(resolvers) => {
                let [head, tail @ ..] = &resolvers[..] else {
                    unreachable!("Composition of resolvers always have at least one element")
                };
                let mut current = head.resolve(ctx, resolver_ctx, last_resolver_value).await?;
                for resolver in tail {
                    current = resolver.resolve(ctx, resolver_ctx, Some(current)).await?;
                }
                Ok(current)
            }
            Resolver::Http(resolver) => {
                resolver
                    .resolve(ctx, resolver_ctx, last_resolver_value.as_ref())
                    .instrument(info_span!("http_resolver", api_name = resolver.api_name))
                    .await
            }
            Resolver::Graphql(resolver) => {
                let runtime_ctx = ctx.data::<runtime::Context>()?;
                let ray_id = runtime_ctx.ray_id();
                let fetch_log_endpoint_url = runtime_ctx.log.fetch_log_endpoint_url.as_deref();

                let registry = ctx.registry();
                let request_headers = ctx.data::<RequestHeaders>().ok();
                let headers = registry
                    .http_headers
                    .get(&format!("GraphQLConnector{}", resolver.name()))
                    .zip(request_headers)
                    .map(|(connector_headers, request_headers)| connector_headers.build_header_vec(request_headers))
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
                    .map(|variable_definition| (&variable_definition.node.name.node, &variable_definition.node))
                    .collect();

                let current_object = resolver_ctx.ty.try_into().ok();

                let target = match resolver.namespace {
                    Some(_) => Target::SelectionSet(Box::new(
                        ctx.item
                            .node
                            .selection_set
                            .node
                            .items
                            .clone()
                            .into_iter()
                            .map(|v| v.node),
                    )),
                    None => Target::Field(ctx.item.clone().into_inner(), resolver_ctx.field.clone()),
                };

                let operation = ctx.query_env.operation.node.ty;
                let error_handler = |error| ctx.add_error(error);
                let variables = ctx.query_env.variables.clone();

                let batcher = &ctx.data::<QueryBatcher>()?;

                resolver
                    .resolve(
                        // Be a lot easier to just pass the context in here...
                        operation,
                        &ray_id,
                        fetch_log_endpoint_url,
                        &headers,
                        fragment_definitions,
                        target,
                        current_object,
                        error_handler,
                        variables,
                        variable_definitions,
                        registry,
                        Some(batcher),
                    )
                    .instrument(info_span!("graphql_resolver", name = resolver.name().as_ref()))
                    .await
                    .map_err(Into::into)
            }
            Resolver::MongoResolver(resolver) => resolver
                .resolve(ctx, resolver_ctx)
                .instrument(info_span!(
                    "mongodb_resolver",
                    operation_type = resolver.operation_type.as_ref(),
                    directive_name = resolver.directive_name,
                    collection = resolver.collection
                ))
                .await
                .map_err(Into::into),
            Resolver::PostgresResolver(resolver) => resolver
                .resolve(ctx, resolver_ctx)
                .instrument(info_span!(
                    "postgresql_resolver",
                    operation = resolver.operation.as_ref(),
                    directive_name = resolver.directive_name
                ))
                .await
                .map_err(Into::into),
            Resolver::FederationEntitiesResolver => resolve_federation_entities(ctx)
                .instrument(info_span!("federation_resolver"))
                .await
                .map_err(Into::into),
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
    PostgresResolver(postgresql::PostgresResolver),
    FederationEntitiesResolver,
}

impl Constraint {
    /// Extracts a ConstraintID for this constraint from the corresponding field
    /// of a `*ByInput` type.
    ///
    /// If the constraint has one field we expect the value to just be a string.
    /// If the constraint has multiple it should be an Object of fieldName: value
    pub fn extract_id_from_by_input_field(&self, ty: &str, value: &ConstValue) -> Option<ConstraintID<'static>> {
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

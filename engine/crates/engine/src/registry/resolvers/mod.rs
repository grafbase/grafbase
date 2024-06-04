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

use engine_parser::types::SelectionSet;
use futures_util::TryFutureExt;
use graphql_cursor::GraphqlCursor;
use registry_v2::MetaField;
use ulid::Ulid;

pub use self::resolved_value::ResolvedValue;
use self::{
    federation::resolve_federation_entities,
    graphql::{QueryBatcher, Target},
    pagination::PaginatedCursor,
};
use super::type_kinds::OutputType;
use crate::{
    registry::connector_headers::build_connector_header_vec, Context, ContextExt, ContextField, Error, RequestHeaders,
};

pub mod atlas_data_api;
pub mod custom;
mod federation;
pub mod graphql;
pub mod http;
mod introspection;
mod logged_fetch;
mod pagination;
pub mod postgres;
mod resolved_value;
pub mod transformer;

use grafbase_tracing::span::resolver::ResolverInvocationSpan;
use grafbase_tracing::span::ResolverInvocationRecorderSpanExt;
use tracing::{info_span, Instrument};

pub use registry_v2::resolvers::Resolver;

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
    pub field: MetaField<'a>,
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

#[async_recursion::async_recursion]
pub(crate) async fn run_resolver(
    resolver: &registry_v2::resolvers::Resolver,
    ctx: &ContextField<'_>,
    resolver_ctx: &ResolverContext<'_>,
    last_resolver_value: Option<ResolvedValue>,
) -> Result<ResolvedValue, Error> {
    use registry_v2::resolvers::Resolver;
    match resolver {
        Resolver::Parent => last_resolver_value.ok_or_else(|| Error::new("No data to propagate!")),
        Resolver::Typename => {
            // This shouldn't really get here, but lets propagate anyway
            last_resolver_value.ok_or_else(|| Error::new("No data to propagate!"))
        }
        Resolver::Transformer(resolver) => transformer::resolve(resolver, ctx, resolver_ctx, last_resolver_value).await,
        Resolver::CustomResolver(resolver) => {
            let resolver_span = ResolverInvocationSpan::new(&resolver.resolver_name).into_span();

            let response: Result<ResolvedValue, Error> = custom::resolve(resolver, ctx, last_resolver_value.as_ref())
                .instrument(resolver_span.clone())
                .inspect_err(|err| {
                    resolver_span.record_failure(&err.message);
                })
                .await;

            response
        }
        Resolver::Composition(resolvers) => {
            let [head, tail @ ..] = &resolvers[..] else {
                unreachable!("Composition of resolvers always have at least one element")
            };
            let mut current = run_resolver(head, ctx, resolver_ctx, last_resolver_value).await?;
            for resolver in tail {
                current = run_resolver(resolver, ctx, resolver_ctx, Some(current)).await?;
            }
            Ok(current)
        }
        Resolver::Http(resolver) => {
            let resolver_span = ResolverInvocationSpan::new(&resolver.api_name).into_span();

            http::resolve(resolver, ctx, resolver_ctx, last_resolver_value)
                .instrument(resolver_span.clone())
                .inspect_err(|err| {
                    resolver_span.record_failure(&err.message);
                })
                .await
        }
        Resolver::Graphql(resolver) => {
            let runtime_ctx = ctx.data::<runtime::Context>()?;
            let ray_id = runtime_ctx.ray_id();
            let fetch_log_endpoint_url = runtime_ctx.log.fetch_log_endpoint_url.as_deref();
            let resolver_span = ResolverInvocationSpan::new(resolver.name().as_ref()).into_span();

            let registry = ctx.registry();
            let request_headers = ctx.data::<RequestHeaders>().ok();
            let headers = registry
                .http_headers
                .get(&format!("GraphQLConnector{}", resolver.name()))
                .zip(request_headers)
                .map(|(connector_headers, request_headers)| {
                    build_connector_header_vec(connector_headers, request_headers)
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
                None => Target::Field(ctx.item.clone().into_inner(), resolver_ctx.field),
            };

            let operation = ctx.query_env.operation.node.ty;
            let error_handler = |error| ctx.add_error(error);
            let variables = ctx.query_env.variables.clone();

            let batcher = ctx.data::<QueryBatcher>().ok();

            graphql::resolve(
                resolver,
                ctx.query_env.futures_spawner.clone(),
                // Be a lot easier to just pass the context in here...
                operation,
                ctx.path.clone(),
                ray_id,
                fetch_log_endpoint_url,
                &headers,
                fragment_definitions,
                target,
                current_object,
                error_handler,
                variables,
                variable_definitions,
                registry,
                batcher,
            )
            .instrument(resolver_span.clone())
            .inspect_err(|err| {
                resolver_span.record_failure(&err.to_string());
            })
            .await
            .map_err(Into::into)
        }
        Resolver::MongoResolver(resolver) => atlas_data_api::resolve(resolver, ctx, resolver_ctx)
            .instrument(info_span!(
                "mongodb_resolver",
                operation_type = resolver.operation_type.as_ref(),
                directive_name = resolver.directive_name,
                collection = resolver.collection
            ))
            .await
            .map_err(Into::into),
        Resolver::PostgresResolver(resolver) => postgres::resolve(resolver, ctx, resolver_ctx)
            .instrument(info_span!(
                "postgres_resolver",
                operation = resolver.operation.as_ref(),
                directive_name = resolver.directive_name
            ))
            .await
            .map_err(Into::into),
        Resolver::FederationEntitiesResolver => resolve_federation_entities(ctx)
            .instrument(info_span!("federation_resolver"))
            .await
            .map_err(Into::into),
        Resolver::Introspection(resolver) => introspection::resolve(resolver, ctx)
            .instrument(info_span!("introspection_resolver"))
            .await
            .map_err(Into::into),
        Resolver::Join(_) => {
            unreachable!("join resolvers should be dealt with in resolver_utils and not get here");
        }
    }
}

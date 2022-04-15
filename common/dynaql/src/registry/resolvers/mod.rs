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

use self::debug::DebugResolver;

use crate::{Context, Error};
use context_data::ContextDataResolver;
use dynamo_mutation::DynamoMutationResolver;
use dynamo_querying::DynamoResolver;
use ulid::Ulid;

pub mod context_data;
pub mod debug;
pub mod dynamo_mutation;
pub mod dynamo_querying;

/// Resolver declarative struct to assign a Resolver for a Field.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Resolver {
    /// Unique id to identify Resolver.
    pub id: Option<String>,
    pub r#type: ResolverType,
}

/// Resolver Context
///
/// Each time a Resolver is accessed to be resolved, a context for the resolving
/// strategy is created.
///
/// This context contain safe access data to be used inside `ResolverTrait`.
/// This give you access to the `resolver_id` which define the resolver, the
/// `execution_id` which is linked to the actual execution, a unique ID is
/// created each time the resolver is called.
pub struct ResolverContext<'a> {
    /// Every declared resolver can have an ID, these ID can be used for
    /// memoization.
    pub resolver_id: Option<&'a str>,
    /// When a resolver is executed, it gains a Resolver unique ID for his
    /// execution, this ID is used for internal cache strategy
    pub execution_id: &'a Ulid,
}

impl<'a> ResolverContext<'a> {
    pub fn new(id: &'a Ulid) -> Self {
        Self {
            resolver_id: None,
            execution_id: id,
        }
    }

    pub fn with_resolver_id(mut self, id: Option<&'a str>) -> Self {
        self.resolver_id = id;
        self
    }
}

#[async_trait::async_trait]
pub trait ResolverTrait: Sync {
    async fn resolve(
        &self,
        ctx: &Context<'_>,
        resolver_ctx: &ResolverContext<'_>,
    ) -> Result<serde_json::Value, Error>;
}

#[async_trait::async_trait]
impl ResolverTrait for Resolver {
    async fn resolve(
        &self,
        ctx: &Context<'_>,
        resolver_ctx: &ResolverContext<'_>,
    ) -> Result<serde_json::Value, Error> {
        match &self.r#type {
            ResolverType::DebugResolver(debug) => debug.resolve(ctx, resolver_ctx).await,
            ResolverType::DynamoResolver(dynamodb) => dynamodb.resolve(ctx, resolver_ctx).await,
            ResolverType::DynamoMutationResolver(dynamodb) => {
                dynamodb.resolve(ctx, resolver_ctx).await
            }
            ResolverType::ContextDataResolver(ctx_data) => {
                ctx_data.resolve(ctx, resolver_ctx).await
            }
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum ResolverType {
    DynamoResolver(DynamoResolver),
    DynamoMutationResolver(DynamoMutationResolver),
    ContextDataResolver(ContextDataResolver),
    DebugResolver(DebugResolver),
}

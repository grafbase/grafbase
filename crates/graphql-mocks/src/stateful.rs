#![allow(clippy::panic)]

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use async_graphql::{Context, EmptySubscription, FieldResult, Object, Schema};

pub struct Stateful {
    schema: Schema<Query, Mutation, EmptySubscription>,
}

impl crate::Subgraph for Stateful {
    fn name(&self) -> String {
        "stateful".to_string()
    }
    async fn start(self) -> crate::MockGraphQlServer {
        crate::MockGraphQlServer::new(self).await
    }
}

impl Default for Stateful {
    fn default() -> Self {
        let state = Arc::new(AtomicUsize::new(0));
        let schema = Schema::build(Query, Mutation, EmptySubscription)
            .enable_federation()
            .data(state)
            .finish();
        Self { schema }
    }
}

#[async_trait::async_trait]
impl super::Schema for Stateful {
    async fn execute(
        &self,
        _headers: Vec<(String, String)>,
        request: async_graphql::Request,
    ) -> async_graphql::Response {
        self.schema.execute(request).await
    }

    fn execute_stream(
        &self,
        request: async_graphql::Request,
        session_data: Option<Arc<async_graphql::Data>>,
    ) -> futures::stream::BoxStream<'static, async_graphql::Response> {
        async_graphql::Executor::execute_stream(&self.schema, request, session_data)
    }

    fn sdl(&self) -> String {
        self.schema
            .sdl_with_options(async_graphql::SDLExportOptions::new().federation())
    }
}

struct Query;

#[Object]
impl Query {
    async fn value(&self, ctx: &Context<'_>) -> usize {
        ctx.data_unchecked::<Arc<AtomicUsize>>().load(Ordering::Relaxed)
    }

    /// Used to test retry logic.
    async fn increment_and_fail_if_less_than(&self, ctx: &Context<'_>, n: usize) -> FieldResult<usize> {
        let state = ctx.data_unchecked::<Arc<AtomicUsize>>();
        let new = state.fetch_add(1, Ordering::Relaxed);
        if new < n {
            // Trigger a 500
            panic!("State value is {new} < {n}")
        } else {
            Ok(new)
        }
    }
}

struct Mutation;

#[Object]
impl Mutation {
    async fn multiply(&self, ctx: &Context<'_>, by: usize) -> usize {
        let state = ctx.data_unchecked::<Arc<AtomicUsize>>();
        state
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| Some(current * by))
            .unwrap();
        state.load(Ordering::Relaxed)
    }

    async fn add(&self, ctx: &Context<'_>, val: usize) -> usize {
        let state = ctx.data_unchecked::<Arc<AtomicUsize>>();
        state
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| Some(current + val))
            .unwrap();
        state.load(Ordering::Relaxed)
    }

    async fn set(&self, ctx: &Context<'_>, val: usize) -> usize {
        let state = ctx.data_unchecked::<Arc<AtomicUsize>>();
        state.store(val, Ordering::Relaxed);
        state.load(Ordering::Relaxed)
    }

    async fn fail(&self) -> async_graphql::FieldResult<usize> {
        Err("This mutation always fails".into())
    }

    async fn faillible(&self) -> Option<async_graphql::FieldResult<usize>> {
        Some(Err("This mutation always fails".into()))
    }

    /// Used to test retry logic.
    async fn increment_and_fail_if_less_than(&self, ctx: &Context<'_>, n: usize) -> FieldResult<usize> {
        let state = ctx.data_unchecked::<Arc<AtomicUsize>>();
        let new = state.fetch_add(1, Ordering::Relaxed);
        if new < n {
            // Trigger a 500
            panic!("State value is {new} < {n}")
        } else {
            Ok(new)
        }
    }
}

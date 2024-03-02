use async_graphql::{EmptySubscription, Schema};
use runtime::cache::Cache;

pub use async_graphql::{Request, Response};

mod error;
pub mod graphql;

pub struct AdminContext {
    pub ray_id: String,
    pub cache: Cache,
}

#[tracing::instrument(skip_all)]
pub async fn execute_admin_request(ctx: AdminContext, request: Request) -> Response {
    Schema::build(graphql::Query, graphql::Mutation::default(), EmptySubscription)
        .data(ctx)
        .finish()
        .execute(request)
        .await
}

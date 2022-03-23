#![allow(clippy::unused_unit)] // for worker::event macro
#![allow(clippy::future_not_send)] // for main

use async_graphql::EmptyMutation;
use async_graphql::EmptySubscription;
use async_graphql::FieldResult;
use async_graphql::Schema;
use async_graphql::ID;
use worker::*;

#[derive(async_graphql::SimpleObject)]
struct Product {
    id: ID,
    price: i32,
}

struct Query;

#[async_graphql::Object]
impl Query {
    async fn product_by_id(&self, id: ID) -> FieldResult<Product> {
        Ok(Product { id, price: 12 })
    }
}

struct WorkerContext {
    schema: Schema<Query, EmptyMutation, EmptySubscription>,
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, context: Context) -> Result<Response> {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();

    let router = Router::with_data(WorkerContext { schema });

    let response = router
        .head_async("/", |_req, _ctx| async move { Response::empty() })
        .options_async("/graphql", |_req, ctx| async move {
            let cors = Cors::new()
                .with_allowed_headers([
                    "Accept",
                    "Authorization",
                    "Content-Type",
                    "Origin",
                    "X-Requested-With",
                ])
                .with_max_age(86400)
                .with_methods([Method::Get, Method::Options, Method::Post])
                .with_origins(["*"]);

            Response::empty()?.with_cors(&cors)
        })
        .post_async("/graphql", |mut req, ctx| async move {
            let schema = ctx.data.schema;
            let cors = Cors::new()
                .with_allowed_headers([
                    "Accept",
                    "Authorization",
                    "Content-Type",
                    "Origin",
                    "X-Requested-With",
                ])
                .with_max_age(86400)
                .with_methods([Method::Get, Method::Options, Method::Post])
                .with_origins(["*"]);

            let gql_req: async_graphql::Request = serde_json::from_str(&req.text().await?)?;
            let gql_res = schema.execute(gql_req).await;

            Response::from_json(&gql_res).and_then(|res| res.with_cors(&cors))
        })
        .run(req, env)
        .await;

    response
}

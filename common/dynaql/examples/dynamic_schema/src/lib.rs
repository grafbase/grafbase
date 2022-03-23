#![allow(clippy::unused_unit)] // for worker::event macro
#![allow(clippy::future_not_send)] // for main

use async_graphql::model::__Schema;
use async_graphql::registry::DebugResolver;
use async_graphql::registry::MetaField;
use async_graphql::registry::MetaInputValue;
use async_graphql::registry::Registry;
use async_graphql::registry::Resolver;
use async_graphql::registry::ResolverType;
use async_graphql::OutputType;
use async_graphql::Schema;
use dynamodb as _;
use dynamodb::DynamoDBContext;
use std::io::Write;
use worker::*;

struct WorkerContext {
    schema: Schema,
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, context: Context) -> Result<Response> {
    // let file = std::fs::read("out.json").unwrap();
    let router = Router::new();
    log::info!("id", "blbl");

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
        .get_async("/sdl", |mut req, ctx| async move {
            let file = ctx
                .kv("AGRIFFON_DEV")?
                .get("SCHEMA")
                .cache_ttl(120)
                .text()
                .await?
                .unwrap();
            let registry: Registry = serde_json::from_str(&file).unwrap();

            let sdl = Schema::build(registry).finish().sdl();

            Response::ok(sdl)
        })
        .post_async("/graphql", |mut req, ctx| async move {
            let file = ctx
                .kv("AGRIFFON_DEV")?
                .get("SCHEMA")
                .cache_ttl(120)
                .text()
                .await?
                .unwrap();
            let registry: Registry = serde_json::from_str(&file).unwrap();
            // TODO: DynamoDB Context into GQL
            let (lat, long) = req.cf().coordinates().expect("can't fail");
            let regions = "eu-central-1,eu-west-1,us-east-1,us-west-1";

            let replication_regions: Vec<aws_region_nearby::AwsRegion> = regions
                .split(',')
                .map(|s| {
                    s.parse()
                        .unwrap_or_else(|_| panic!("replication region name `{}` is invalid", s))
                })
                .collect();

            let db_context = DynamoDBContext::new(
                "trace_id",
                "AKIAZDU6IPETLZHZT4M4".into(),
                "kOktlcn6eLLnPlSPIH6zQI/1ZjJC4jeiFpDT6Ftl".into(),
                replication_regions,
                "grafbase-dev-dynamodb-grafbaseCE0A2685-1VPT6XNVNJ3IS",
                lat,
                long,
            );

            let schema = Schema::build(registry).data(db_context).finish();
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

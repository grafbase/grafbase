#![allow(unused_crate_dependencies)]

use std::future::IntoFuture;

use divan::AllocProfiler;

use graphql_mocks::{LargeResponseSchema, MockGraphQlServer};
use integration_tests::{EngineBuilder, ResponseExt};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

#[divan::bench]
fn ton_of_data_test(bencher: divan::Bencher<'_, '_>) {
    bencher
        .with_inputs(|| {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let graphql_mock = runtime.block_on(MockGraphQlServer::new(LargeResponseSchema));

            let engine = runtime.block_on(EngineBuilder::new(schema(graphql_mock.port(), true)).build());

            (runtime, graphql_mock, engine)
        })
        .bench_values(|(runtime, graphql_mock, engine)| {
            let result = runtime.block_on(engine.execute(QUERY).into_future());

            let result = result.assert_success();

            // Return all our inputs so their Drop doesn't get counted in our bench
            (result, graphql_mock, engine, runtime)
        });
}

const QUERY: &str = "
    query {
        gothub {
            pullRequestsAndIssues {
                __typename
                title
                ... on GothubPullRequest {
                    id
                    title
                    checks
                    author {
                        __typename
                        ... on GothubUser {
                            name
                            email
                        }
                    }
                    status
                }
                ... on GothubIssue {
                    title
                    author {
                        __typename
                        ... on GothubUser {
                            name
                            email
                        }
                    }
                }
            }
        }
    }
";

fn schema(port: u16, namespace: bool) -> String {
    format!(
        r#"
          extend schema
          @graphql(
            name: "gothub",
            namespace: {namespace},
            url: "http://127.0.0.1:{port}",
          )
        "#
    )
}

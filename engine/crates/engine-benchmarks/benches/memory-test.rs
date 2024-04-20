#![allow(unused_crate_dependencies)]

use std::future::IntoFuture;

// use divan::AllocProfiler;

use engine_benchmarks::MockProcess;
use integration_tests::{EngineBuilder, ResponseExt};
use serde_json::json;

// #[global_allocator]
// static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    divan::main();
}

#[divan::bench]
fn ton_of_data_test(bencher: divan::Bencher<'_, '_>) {
    let mock = MockProcess::new();
    let port = mock.port;

    let result = serde_json::from_str(
        &std::fs::read_to_string("/Users/graeme/src/grafbase/tripadvisor-repro/parse-result.json").unwrap(),
    )
    .unwrap();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let engine = runtime.block_on(
        EngineBuilder::new(schema(port, true))
            .with_forced_parse_result(result)
            .build(),
    );

    bencher
        .with_inputs(|| {
            // Make a new runtime per test to make sure nothing interferes
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            runtime
        })
        .bench_values(|runtime| {
            let result = runtime.block_on(
                engine
                    .execute(QUERY)
                    .variables(json!(
                        {
                            "listId": 67,
                            "countryId": 1
                        }
                    ))
                    .into_future(),
            );

            let result = result.assert_success();

            // Return all our inputs so their Drop doesn't get counted in our bench
            (result, runtime)
        });
}

const QUERY: &str = "
query ShelfItems($listId: Float!, $countryId: Float!) {
    db {
      listItems(listId: $listId, countryId: $countryId, limit: 9) {
        id
        title
        image
        subjectId
        subjectReferenceId
        destination {
          id
          slug
        }
        departurePort {
          id
          name
        }
        port {
          id
          slug
        }
        ship {
          id
          slug
        }
        cruiseLine {
          id
          slug
        }
      }
    }
  }
";
#[cfg(NOPE)]
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

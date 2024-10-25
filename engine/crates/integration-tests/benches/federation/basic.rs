use criterion::Criterion;
use integration_tests::{federation::DeterministicEngine, runtime};
use serde_json::json;

use crate::federation::SCHEMA;

pub fn without_operation_cache(c: &mut Criterion) {
    let engine = prepare(false);

    c.bench_function("basic_without_operation_cache", |b| {
        // Insert a call to `to_async` to convert the bencher to async mode.
        // The timing loops are the same as with the normal bencher.
        b.to_async(runtime()).iter(|| engine.raw_execute());
    });
}

pub fn with_operation_cache(c: &mut Criterion) {
    let engine = prepare(true);
    c.bench_function("basic", |b| {
        // Insert a call to `to_async` to convert the bencher to async mode.
        // The timing loops are the same as with the normal bencher.
        b.to_async(runtime()).iter(|| engine.raw_execute());
    });
}

fn prepare(operation_cache: bool) -> DeterministicEngine {
    runtime().block_on(async {
    const QUERY: &str = r#"
            query ExampleQuery {
                me {
                    id
                    username
                    reviews {
                        body
                        product {
                            reviews {
                                author {
                                    id
                                    username
                                }
                                body
                            }
                        }
                    }
                }
            }
            "#;
    let mut builder = DeterministicEngine::builder(SCHEMA, QUERY);

    if !operation_cache {
        builder = builder.without_operation_cache();
    }

    let engine = builder
        .with_subgraph_response(json!({"data":{"me":{"id":"1234","username":"Me"}}}))
        .with_subgraph_response(                json!({"data":{"_entities":[{"__typename":"User","reviews":[{"body":"A highly effective form of birth control.","product":{"reviews":[{"author":{"id":"1234"},"body":"A highly effective form of birth control."}]}},{"body":"Fedoras are one of the most fashionable hats around and can look great with a variety of outfits.","product":{"reviews":[{"author":{"id":"1234"},"body":"Fedoras are one of the most fashionable hats around and can look great with a variety of outfits."}]}}]}]}}))
        .with_subgraph_response(                json!({"data":{"_entities":[{"__typename":"User","username":"Me"},{"__typename":"User","username":"Me"}]}})
)

        .build()
        .await;

    let response = engine.execute().await;

    // Sanity check it works.
    insta::assert_json_snapshot!(response);

    engine

    })
}

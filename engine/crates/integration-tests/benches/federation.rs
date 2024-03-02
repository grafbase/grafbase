#![allow(unused_crate_dependencies)]

use criterion::{criterion_group, criterion_main, Criterion};
use engine::ResponseBody;
use integration_tests::federation::FederationGatewayWithoutIO;
use serde_json::json;

const SCHEMA: &str = include_str!("../data/federated-graph-schema.graphql");
const PATHFINDER_INTROSPECTION_QUERY: &str = include_str!("../data/introspection.graphql");

#[allow(clippy::panic)]
pub fn introspection(c: &mut Criterion) {
    let bench = FederationGatewayWithoutIO::new(SCHEMA, PATHFINDER_INTROSPECTION_QUERY, &[json!({"data": null})]);
    let response = integration_tests::runtime().block_on(bench.raw_execute());
    let ResponseBody::Bytes(bytes) = response.body else {
        panic!("expected bytes");
    };

    // Sanity check it works.
    insta::assert_snapshot!(String::from_utf8_lossy(&bytes));

    c.bench_function("introspection", |b| {
        // Insert a call to `to_async` to convert the bencher to async mode.
        // The timing loops are the same as with the normal bencher.
        b.to_async(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
        )
        .iter(|| bench.raw_execute());
    });
}

#[allow(clippy::panic)]
pub fn basic_federation(c: &mut Criterion) {
    let bench = FederationGatewayWithoutIO::new(
        SCHEMA,
        r#"
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
        "#,
        &[
            json!({"data":{"me":{"id":"1234","username":"Me"}}}),
            json!({"data":{"_entities":[{"__typename":"User","reviews":[{"body":"A highly effective form of birth control.","product":{"reviews":[{"author":{"id":"1234"},"body":"A highly effective form of birth control."}]}},{"body":"Fedoras are one of the most fashionable hats around and can look great with a variety of outfits.","product":{"reviews":[{"author":{"id":"1234"},"body":"Fedoras are one of the most fashionable hats around and can look great with a variety of outfits."}]}}]}]}}),
            json!({"data":{"_entities":[{"__typename":"User","username":"Me"},{"__typename":"User","username":"Me"}]}}),
        ],
    );
    let response = integration_tests::runtime().block_on(bench.raw_execute());
    let ResponseBody::Bytes(bytes) = response.body else {
        panic!("expected bytes");
    };

    // Sanity check it works.
    insta::assert_snapshot!(String::from_utf8_lossy(&bytes));

    c.bench_function("basic_federation", |b| {
        // Insert a call to `to_async` to convert the bencher to async mode.
        // The timing loops are the same as with the normal bencher.
        b.to_async(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
        )
        .iter(|| bench.raw_execute());
    });
}

criterion_group!(benches, introspection, basic_federation);
criterion_main!(benches);

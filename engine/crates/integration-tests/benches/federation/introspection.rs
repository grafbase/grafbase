use criterion::Criterion;
use integration_tests::{federation::DeterministicEngine, runtime};

use crate::federation::SCHEMA;

const PATHFINDER_INTROSPECTION_QUERY: &str = include_str!("../../data/introspection.graphql");

pub fn without_operation_cache(c: &mut Criterion) {
    let engine = prepare(false);

    c.bench_function("introspection_without_operation_cache", |b| {
        // Insert a call to `to_async` to convert the bencher to async mode.
        // The timing loops are the same as with the normal bencher.
        b.to_async(runtime()).iter(|| engine.raw_execute());
    });
}

pub fn with_operation_cache(c: &mut Criterion) {
    let engine = prepare(true);

    c.bench_function("introspection", |b| {
        // Insert a call to `to_async` to convert the bencher to async mode.
        // The timing loops are the same as with the normal bencher.
        b.to_async(runtime()).iter(|| engine.raw_execute());
    });
}

fn prepare(operation_cache: bool) -> DeterministicEngine {
    runtime().block_on(async {
        let mut builder = DeterministicEngine::builder(SCHEMA, PATHFINDER_INTROSPECTION_QUERY);

        if !operation_cache {
            builder = builder.without_operation_cache();
        }

        let engine = builder.build().await;
        let response = engine.execute().await;

        // Sanity check it works.
        insta::assert_json_snapshot!(response);

        engine
    })
}

mod basic;
mod complex_shape;
mod introspection;
mod query_planning;

use criterion::{Criterion, criterion_group};
use pprof::criterion::{Output, PProfProfiler};

const SCHEMA: &str = include_str!("../../data/federated-graph-schema.graphql");

criterion_group! {
    name = federation;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(None)));
    targets = introspection::with_operation_cache, introspection::without_operation_cache, basic::with_operation_cache, basic::without_operation_cache, complex_shape::complex_schema, complex_shape::complex_query, query_planning::query_plan1
}

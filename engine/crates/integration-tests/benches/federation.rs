#![allow(unused_crate_dependencies)]
use std::{fmt::Write, sync::OnceLock};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use futures::{stream::FuturesUnordered, StreamExt};
use indoc::{indoc, writedoc};
use integration_tests::{federation::DeterministicEngine, runtime};
use itertools::Itertools;
use pprof::criterion::{Output, PProfProfiler};
use serde_json::json;

const SCHEMA: &str = include_str!("../data/federated-graph-schema.graphql");
const PATHFINDER_INTROSPECTION_QUERY: &str = include_str!("../data/introspection.graphql");

pub fn introspection(c: &mut Criterion) {
    let bench = runtime().block_on(DeterministicEngine::new(
        SCHEMA,
        PATHFINDER_INTROSPECTION_QUERY,
        &[json!({"data": null})],
    ));
    let response = runtime().block_on(bench.execute());

    // Sanity check it works.
    insta::assert_json_snapshot!(response);

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

pub fn basic_federation(c: &mut Criterion) {
    let bench = runtime().block_on(DeterministicEngine::new(
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
        ));
    let response = runtime().block_on(bench.execute());

    // Sanity check it works.
    insta::assert_json_snapshot!(response);

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

pub fn complex_schema(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_schema");

    for (size, case) in ComplexSchemaAndQuery::cases() {
        group.throughput(Throughput::Bytes(case.schema.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.to_async(
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap(),
            )
            .iter(|| case.to_engine())
        });
    }
    group.finish();
}

pub fn cynic_complex_schema(c: &mut Criterion) {
    let mut group = c.benchmark_group("cynic_schema");

    for (size, case) in ComplexSchemaAndQuery::cases() {
        group.throughput(Throughput::Bytes(case.schema.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| cynic_parser::parse_type_system_document(&case.schema).unwrap())
        });
    }
    group.finish();
}

pub fn complex_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_query");

    let cases = runtime().block_on(
        ComplexSchemaAndQuery::cases()
            .iter()
            .map(|(size, case)| async { (*size, case.query.len(), case.to_engine().await) })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>(),
    );

    // Sanity check it works.
    let response = runtime().block_on(async { cases.first().unwrap().2.execute().await });
    insta::assert_json_snapshot!(response);

    for (size, query_len, engine) in cases {
        group.throughput(Throughput::Bytes(query_len as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.to_async(
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap(),
            )
            .iter(|| engine.raw_execute());
        });
    }
    group.finish();
}

struct ComplexSchemaAndQuery {
    schema: String,
    query: String,
}

impl ComplexSchemaAndQuery {
    fn cases() -> &'static [(usize, ComplexSchemaAndQuery)] {
        static CACHE: OnceLock<Vec<(usize, ComplexSchemaAndQuery)>> = OnceLock::new();
        CACHE.get_or_init(|| {
            [8, 16, 32, 48]
                .into_iter()
                .map(|size| {
                    let case = ComplexSchemaAndQuery::build(size, 2, size * size);
                    println!(
                        "Case {size}: schema: {} KB / query: {} KB",
                        case.schema.len() >> 10,
                        case.query.len() >> 10
                    );
                    (size, case)
                })
                .collect::<Vec<_>>()
        })
    }

    fn build(n: usize, k: usize, extras: usize) -> ComplexSchemaAndQuery {
        let mut schema = indoc!(
            r###"
            enum join__Graph {
              SUB @join__graph(name: "sub", url: "http://127.0.0.1:46697")
            }

            type Query {
                node: Node @join__field(graph: SUB)
            }

            interface Node {
                id: ID!
                node: Node!
            }

            type Nothing implements Node {
                id: ID!
                node: Node!
            }
            "###
        )
        .to_string();

        let mut query = indoc!(
            r###"
            query {
              node {
                __typename
                id
                ...Complex
                node {
                  __typename
                  id
                  ...Complex
                }
              }
            }

            fragment Complex on Node {
            "###
        )
        .to_string();

        let mut interface_to_fields = (0..n).map(|i| vec![i]).collect::<Vec<_>>();
        for (j, (i1, i2)) in (0..n).tuple_combinations().enumerate() {
            interface_to_fields[i1].push(n + j);
            interface_to_fields[i2].push(n + j);
        }

        for (i, fields) in interface_to_fields.iter().enumerate() {
            writedoc!(
                schema,
                r###"
                interface I{i} implements Node {{
                    id: ID! @join__field(graph: SUB)
                    node: Node! @join__field(graph: SUB)
                {}
                }}
                "###,
                fields.iter().format_with("\n", |j, f| {
                    f(&format_args!("    f{}: String! @join_field(graph: SUB)", j))
                })
            )
            .unwrap();
            writedoc!(
                query,
                "  ... on I{} {{ {} }}\n",
                i,
                fields.iter().format_with(" ", |j, f| { f(&format_args!("f{}", j)) })
            )
            .unwrap();
        }

        let mut buffer: Vec<usize> = Vec::new();
        for c in 1..=n.min(k).min(2) {
            for (j, interfaces) in (0..n).combinations(c).enumerate() {
                for i in &interfaces {
                    buffer.extend_from_slice(&interface_to_fields[*i])
                }
                buffer.sort_unstable();
                writedoc!(
                    schema,
                    r###"
                    type T{c}x{j} implements Node & {} {{
                        id: ID! @join__field(graph: SUB)
                        node: Node!
                    {}
                    }}
                    "###,
                    interfaces.iter().format_with(" & ", |i, f| f(&format_args!("I{}", i))),
                    buffer.drain(..).dedup().format_with("\n", |i, f| f(&format_args!(
                        "    f{i}: String! @join_field(graph: SUB)"
                    )))
                )
                .unwrap();
            }
        }

        for i in 0..extras {
            writedoc!(
                schema,
                r###"
                type E{i} implements Node {{
                    id: ID! @join__field(graph: SUB)
                    node: Node!
                }}
                "###,
            )
            .unwrap();
        }
        query.push('}');

        ComplexSchemaAndQuery { schema, query }
    }

    async fn to_engine(&self) -> DeterministicEngine {
        DeterministicEngine::builder(&self.schema, &self.query)
            .without_hot_cache()
            .with_subgraph_response(json!({"data":{"node":{"id":"1234", "__typename": "Nothing"}}}))
            .build()
            .await
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(1000, Output::Flamegraph(None)));
    targets = introspection, basic_federation, complex_schema, complex_query, cynic_complex_schema
}

criterion_main!(benches);

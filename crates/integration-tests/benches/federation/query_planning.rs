use std::{fmt::Write, ops::Range, sync::OnceLock};

use criterion::{BenchmarkId, Criterion, Throughput};
use futures::{StreamExt, stream::FuturesUnordered};
use indoc::writedoc;
use integration_tests::{federation::DeterministicEngine, runtime};
use itertools::{Combinations, Itertools};
use serde_json::json;

pub fn query_plan1(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_plan1");

    let cases = runtime().block_on(
        SchemaAndQuery::cases()
            .iter()
            .map(|(params, case)| async { (*params, case.query.len(), case.to_engine().await) })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>(),
    );

    // Sanity check it works.
    let response = runtime().block_on(async { cases.first().unwrap().2.execute().await });
    insta::assert_json_snapshot!(response);

    for (params, query_len, engine) in cases {
        group.throughput(Throughput::Bytes(query_len as u64));
        group.bench_with_input(BenchmarkId::from_parameter(params), &params, |b, _| {
            b.to_async(runtime()).iter(|| engine.raw_execute());
        });
    }
    group.finish();
}

struct SchemaAndQuery {
    schema: String,
    query: String,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Params {
    n_subgraphs: usize,
    n_fields: usize,
    depth: usize,
    keys: usize,
}

impl std::fmt::Display for Params {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}:{}-{}", self.n_subgraphs, self.keys, self.n_fields, self.depth)
    }
}

impl SchemaAndQuery {
    fn cases() -> &'static [(Params, SchemaAndQuery)] {
        static CACHE: OnceLock<Vec<(Params, SchemaAndQuery)>> = OnceLock::new();
        CACHE.get_or_init(|| {
            [
                Params {
                    n_subgraphs: 3,
                    n_fields: 2,
                    depth: 2,
                    keys: 1,
                },
                Params {
                    n_subgraphs: 4,
                    n_fields: 3,
                    depth: 4,
                    keys: 1,
                },
                Params {
                    n_subgraphs: 5,
                    n_fields: 4,
                    depth: 4,
                    keys: 1,
                },
                Params {
                    n_subgraphs: 7,
                    n_fields: 4,
                    depth: 4,
                    keys: 1,
                },
                Params {
                    n_subgraphs: 9,
                    n_fields: 3,
                    depth: 4,
                    keys: 3,
                },
            ]
            .into_iter()
            .map(|params| {
                let case = SchemaAndQuery::build(params);
                println!(
                    "Case {}: schema: {} KB / query: {} KB",
                    params,
                    case.schema.len() >> 10,
                    case.query.len() >> 10
                );

                (params, case)
            })
            .collect::<Vec<_>>()
        })
    }

    fn build(params: Params) -> SchemaAndQuery {
        use rand::prelude::*;

        assert!(params.n_fields <= params.n_subgraphs);

        let mut schema = String::from("enum join__Graph {\n");
        for i in 0..params.n_subgraphs {
            writeln!(
                schema,
                r#"  SUB{i} @join__graph(name: "sub{i}", url: "http://localhost:100{i}")"#
            )
            .unwrap();
        }
        schema.push_str("}\n\n");
        let mut rng = StdRng::seed_from_u64(78);

        let mut fields_subgraphs = (0..params.n_subgraphs)
            .combinations(params.n_fields)
            .collect::<Vec<_>>();
        fields_subgraphs.shuffle(&mut rng);
        writedoc!(
            schema,
            r###"
            type Query {} {{
                node: Node
            }}
            type Node {} {{
            {}
            {}
            }}
            "###,
            (0..params.n_subgraphs).format_with(" ", |i, f| f(&format_args!(r#"@join__type(graph: SUB{i})"#))),
            (0..params.keys)
                .cartesian_product(0..params.n_subgraphs)
                .format_with(" ", |(key, subraph), f| f(&format_args!(
                    r#"@join__type(graph: SUB{subraph}, key: "id{key}")"#
                ))),
            (0..params.keys).format_with("\n", |key, f| f(&format_args!("    id{key}: ID!"))),
            fields_subgraphs
                .into_iter()
                .enumerate()
                .format_with("\n", |(i, subgraphs), f| {
                    let join_fields = format!(
                        "{}",
                        subgraphs
                            .into_iter()
                            .format_with(" ", |i, f| f(&format_args!("@join__field(graph: SUB{i})")))
                    );
                    f(&format_args!(
                        "    n{i}: Node {join_fields}\n    f{i}: String {join_fields}",
                    ))
                }),
        )
        .unwrap();

        let mut query = String::from("query {\n  node");
        let mut combinations: Combinations<Range<usize>> = (0..params.n_subgraphs).combinations(params.n_fields);
        Self::write_selection_set(params, &mut query, &mut combinations, 1, params.depth);
        query.push_str("}\n");

        SchemaAndQuery { schema, query }
    }

    fn write_selection_set(
        params: Params,
        query: &mut String,
        combinations: &mut Combinations<Range<usize>>,
        indent: usize,
        depth: usize,
    ) {
        query.push_str(" {\n");
        let combination = match combinations.next() {
            Some(c) => c,
            None => {
                *combinations = (0..params.n_subgraphs).combinations(params.n_fields);
                combinations.next().unwrap()
            }
        };
        for i in combination {
            if depth != 0 {
                write!(query, "{}n{i}", "  ".repeat(indent + 1)).unwrap();
                Self::write_selection_set(params, query, combinations, indent + 1, depth - 1)
            } else {
                writeln!(query, "{}f{i}", "  ".repeat(indent + 1)).unwrap();
            }
        }
        write!(query, "{}", "  ".repeat(indent)).unwrap();
        query.push_str("}\n");
    }

    async fn to_engine(&self) -> DeterministicEngine {
        let mut engine = DeterministicEngine::builder(&self.schema, &self.query).without_operation_cache();
        // As many as maximum number of subgraphs
        for _ in 0..100 {
            engine = engine.with_subgraph_response(json!(
                {"data":{"node":null}}
            ));
        }
        engine.build().await
    }
}

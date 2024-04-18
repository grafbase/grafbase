#![allow(unused_crate_dependencies)]
use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(criterion: &mut Criterion) {
    let schema = r#"
        extend schema
            @graphql(
                name: "SW1"
                namespace: true
                url: "https://swapi-graphql.netlify.app/.netlify/functions/index"
            )
            @graphql(
                name: "SW2"
                namespace: true
                url: "https://swapi-graphql.netlify.app/.netlify/functions/index"
            )
            @graphql(
                name: "SW3"
                namespace: true
                url: "https://swapi-graphql.netlify.app/.netlify/functions/index"
            )
            @graphql(
                name: "SW4"
                namespace: true
                url: "https://swapi-graphql.netlify.app/.netlify/functions/index"
            )
            @graphql(
                name: "SW5"
                namespace: true
                url: "https://swapi-graphql.netlify.app/.netlify/functions/index"
            )
            @graphql(
                name: "SW6"
                namespace: true
                url: "https://swapi-graphql.netlify.app/.netlify/functions/index"
            )
            @graphql(
                name: "SW7"
                namespace: true
                url: "https://swapi-graphql.netlify.app/.netlify/functions/index"
            )
            @graphql(
                name: "SW8"
                namespace: true
                url: "https://swapi-graphql.netlify.app/.netlify/functions/index"
            )
    "#;

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let env = std::collections::HashMap::new();
    let mut group = criterion.benchmark_group("parse_sdl() with IO");

    group.sample_size(15).bench_function("parse_sdl", |b| {
        b.iter(|| {
            runtime
                .block_on(grafbase_local_server::parse_sdl(schema, &env))
                .unwrap()
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

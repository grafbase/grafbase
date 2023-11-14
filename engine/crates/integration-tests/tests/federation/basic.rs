//! Tests of Basic GraphQL things going through our federation setup.
//!
//! This file shouldn't have much federation specific stuff in it, mostly just checking
//! that our engine supports all the things a normal GraphQL server should.

use engine_v2::Engine;
use integration_tests::{
    mocks::graphql::{FakeGithubSchema, Schema},
    runtime,
};

#[test]
#[ignore]
fn single_field_from_single_server() {
    let schema = async_graphql_parser::parse_schema(FakeGithubSchema.sdl()).unwrap();

    let mut subgraphs = graphql_composition::Subgraphs::default();
    subgraphs.ingest(&schema, "schema");
    let graph = graphql_composition::compose(&subgraphs).into_result().unwrap();

    let engine = Engine::new(graph.into());

    let query = engine_parser::parse_query("query { serverVersion }")
        .unwrap()
        .operations
        .iter()
        .next()
        .unwrap()
        .1
        .node
        .clone();

    let response = runtime().block_on(engine.execute(query)).unwrap();

    insta::assert_json_snapshot!(response, @"");
}

#[test]
#[ignore]
fn named_fragment() {
    todo!("write this")
}

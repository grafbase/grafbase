//! Tests of Basic GraphQL things going through our federation setup.
//!
//! This file shouldn't have much federation specific stuff in it, mostly just checking
//! that our engine supports all the things a normal GraphQL server should.

mod fragments;
mod variables;

use engine_v2::Engine;
use integration_tests::{federation::EngineV2Ext, mocks::graphql::FakeGithubSchema, runtime};

#[test]
#[ignore]
fn single_field_from_single_server() {
    let engine = Engine::build().with_schema("schema", FakeGithubSchema).finish();

    let response = runtime()
        .block_on(async move { engine.execute("query { serverVersion }").await })
        .unwrap();

    insta::assert_json_snapshot!(response, @"");
}

#[test]
#[ignore]
fn test_introspection_matches() {
    todo!("introspect fake server and introspect federation server - schemas should match")
}

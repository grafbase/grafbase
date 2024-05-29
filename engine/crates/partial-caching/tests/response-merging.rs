#![allow(unused_crate_dependencies, clippy::panic)]

use common_types::auth::ExecutionAuth;
use graph_entities::QueryResponse;
use http::HeaderMap;
use insta::assert_json_snapshot;
use partial_caching::FetchPhaseResult;
use runtime::cache::Entry;
use serde::Deserialize;
use serde_json::json;

const SCHEMA: &str = r#"
    type Query {
        user: User @resolver(name: "whatever")
    }

    type User {
        name: String @cache(maxAge: 140)
        email: String @cache(maxAge: 130)
        someConstant: String @cache(maxAge: 120)
        nested: Nested
    }

    type Nested {
        someThing: String @cache(maxAge: 160)
    }
"#;

const QUERY: &str = r#"query { user { name email someConstant nested { someThing }}}"#;

#[test]
fn test_simple_response_merging() {
    let registry = build_registry(SCHEMA);
    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();
    let mut fetch_phase = plan.start_fetch_phase(&auth(), &headers(), &variables());

    let cache_keys = fetch_phase.cache_keys();
    fetch_phase.record_cache_entry(&cache_keys[0], Entry::Hit(json!({"user": {"name": "Jane"}})));
    fetch_phase.record_cache_entry(
        &cache_keys[3],
        Entry::Hit(json!({"user": {"nested": {"someThing": "hello"}}})),
    );

    let FetchPhaseResult::PartialHit(execution) = fetch_phase.finish() else {
        panic!("We didn't hit everything so this should always be a partial");
    };

    let mut query_response = QueryResponse::default();
    let root_node = query_response.from_serde_value(json!({"user": {"email": "whatever", "someConstant": "123"}}));
    query_response.set_root_unchecked(root_node);

    let (response, _updates) = execution.handle_response(query_response, false);

    // Note: This is technically wrong because the order of the fields doesn't match the query.
    // Have raised GB-6813 to look into this (or not, we'll see)
    assert_json_snapshot!(response.as_graphql_data(), @r###"
    {
      "user": {
        "email": "whatever",
        "someConstant": "123",
        "name": "Jane",
        "nested": {
          "someThing": "hello"
        }
      }
    }
    "###);
}

#[test]
fn test_handles_nulls_gracefull() {
    let registry = build_registry(SCHEMA);
    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();
    let mut fetch_phase = plan.start_fetch_phase(&auth(), &headers(), &variables());

    let cache_keys = fetch_phase.cache_keys();
    fetch_phase.record_cache_entry(&cache_keys[0], Entry::Hit(json!({"user": {"name": "Jane"}})));
    fetch_phase.record_cache_entry(
        &cache_keys[3],
        Entry::Hit(json!({"user": {"nested": {"someThing": "hello"}}})),
    );

    let FetchPhaseResult::PartialHit(execution) = fetch_phase.finish() else {
        panic!("We didn't hit everything so this should always be a partial");
    };

    let mut query_response = QueryResponse::default();
    let root_node = query_response.from_serde_value(json!({"user": null}));
    query_response.set_root_unchecked(root_node);

    let (response, _updates) = execution.handle_response(query_response, false);

    assert_json_snapshot!(response.as_graphql_data(), @r###"
    {
      "user": null
    }
    "###);
}

#[test]
fn test_complete_hit() {
    let registry = build_registry(SCHEMA);
    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();
    let mut fetch_phase = plan.start_fetch_phase(&auth(), &headers(), &variables());

    let cache_keys = fetch_phase.cache_keys();
    fetch_phase.record_cache_entry(&cache_keys[0], Entry::Hit(json!({"user": {"name": "Jane"}})));
    fetch_phase.record_cache_entry(&cache_keys[1], Entry::Hit(json!({"user": {"email": "whatever"}})));
    fetch_phase.record_cache_entry(&cache_keys[2], Entry::Hit(json!({"user": {"someConstant": "123"}})));
    fetch_phase.record_cache_entry(
        &cache_keys[3],
        Entry::Hit(json!({"user": {"nested": {"someThing": "hello"}}})),
    );

    let FetchPhaseResult::CompleteHit(hit) = fetch_phase.finish() else {
        panic!("We hit everything so this should always be a complete hit");
    };

    let (response, _updates) = hit.response_and_updates();

    assert_json_snapshot!(response.as_graphql_data(), @r###"
    {
      "user": {
        "name": "Jane",
        "email": "whatever",
        "someConstant": "123",
        "nested": {
          "someThing": "hello"
        }
      }
    }
    "###);
}

fn build_registry(schema: &str) -> registry_for_cache::PartialCacheRegistry {
    registry_upgrade::convert_v1_to_partial_cache_registry(parser_sdl::parse_registry(schema).unwrap()).unwrap()
}

fn auth() -> ExecutionAuth {
    ExecutionAuth::ApiKey
}

fn headers() -> HeaderMap {
    http::HeaderMap::new()
}

fn variables() -> engine_value::Variables {
    engine_value::Variables::deserialize(json!({})).unwrap()
}

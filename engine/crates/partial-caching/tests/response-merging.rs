#![allow(unused_crate_dependencies, clippy::panic)]

use std::time::Duration;

use common_types::auth::ExecutionAuth;
use graph_entities::QueryResponse;
use headers::HeaderMapExt;
use http::{HeaderMap, HeaderValue};
use insta::assert_json_snapshot;
use partial_caching::{type_relationships::no_subtypes, FetchPhaseResult};
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
        noncacheable: String
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
    fetch_phase.record_cache_entry(&cache_keys[0], hit(json!({"user": {"name": "Jane"}}), 10));
    fetch_phase.record_cache_entry(
        &cache_keys[3],
        hit(json!({"user": {"nested": {"someThing": "hello"}}}), 10),
    );

    let FetchPhaseResult::PartialHit(execution) = fetch_phase.finish(no_subtypes()) else {
        panic!("We didn't hit everything so this should always be a partial");
    };

    let mut query_response = QueryResponse::default();
    let root_node = query_response.from_serde_value(json!({"user": {"email": "whatever", "someConstant": "123"}}));
    query_response.set_root_unchecked(root_node);

    let (response, _updates) = execution.handle_full_response(query_response, false);

    // Note: This is technically wrong because the order of the fields doesn't match the query.
    // Have raised GB-6813 to look into this (or not, we'll see)
    assert_json_snapshot!(response.body.as_graphql_data(), @r###"
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
    fetch_phase.record_cache_entry(&cache_keys[0], hit(json!({"user": {"name": "Jane"}}), 10));
    fetch_phase.record_cache_entry(
        &cache_keys[3],
        hit(json!({"user": {"nested": {"someThing": "hello"}}}), 10),
    );

    let FetchPhaseResult::PartialHit(execution) = fetch_phase.finish(no_subtypes()) else {
        panic!("We didn't hit everything so this should always be a partial");
    };

    let mut query_response = QueryResponse::default();
    let root_node = query_response.from_serde_value(json!({"user": null}));
    query_response.set_root_unchecked(root_node);

    let (response, _updates) = execution.handle_full_response(query_response, false);

    assert_json_snapshot!(response.body.as_graphql_data(), @r###"
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
    fetch_phase.record_cache_entry(&cache_keys[0], hit(json!({"user": {"name": "Jane"}}), 10));
    fetch_phase.record_cache_entry(&cache_keys[1], hit(json!({"user": {"email": "whatever"}}), 100));
    fetch_phase.record_cache_entry(&cache_keys[2], hit(json!({"user": {"someConstant": "123"}}), 200));
    fetch_phase.record_cache_entry(
        &cache_keys[3],
        hit(json!({"user": {"nested": {"someThing": "hello"}}}), 10),
    );

    let FetchPhaseResult::CompleteHit(hit) = fetch_phase.finish(no_subtypes()) else {
        panic!("We hit everything so this should always be a complete hit");
    };

    let (response, _updates) = hit.response_and_updates();

    assert_eq!(
        response.headers.get("x-grafbase-cache"),
        Some(&HeaderValue::from_static("HIT"))
    );

    let response_cache_control = response.headers.typed_get::<headers::CacheControl>().unwrap();
    assert_eq!(response_cache_control.max_age().unwrap().as_secs(), 10);

    assert_json_snapshot!(response.body.as_graphql_data(), @r###"
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

#[test]
fn test_partial_hit_when_lowest_max_age_is_hit() {
    let registry = build_registry(SCHEMA);
    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();
    let mut fetch_phase = plan.start_fetch_phase(&auth(), &headers(), &variables());

    let cache_keys = fetch_phase.cache_keys();
    fetch_phase.record_cache_entry(&cache_keys[0], hit(json!({"user": {"name": "Jane"}}), 10));

    let FetchPhaseResult::PartialHit(execution) = fetch_phase.finish(no_subtypes()) else {
        panic!("We didn't hit everything so this should always be a partial");
    };

    let mut query_response = QueryResponse::default();
    let root_node = query_response.from_serde_value(json!({
        "user": {
            "email": "whatever",
            "someConstant": "123",
            "nested": {"someThing": "hello"}
        }
    }));
    query_response.set_root_unchecked(root_node);

    let (response, _updates) = execution.handle_full_response(query_response, false);

    assert_eq!(
        response.headers.get("x-grafbase-cache"),
        Some(&HeaderValue::from_static("PARTIAL_HIT"))
    );

    let response_cache_control = response.headers.typed_get::<headers::CacheControl>().unwrap();
    assert_eq!(response_cache_control.max_age().unwrap().as_secs(), 10);
}

#[test]
fn test_partial_hit_when_lowest_max_age_is_miss() {
    let registry = build_registry(SCHEMA);
    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();
    let mut fetch_phase = plan.start_fetch_phase(&auth(), &headers(), &variables());

    let cache_keys = fetch_phase.cache_keys();
    fetch_phase.record_cache_entry(&cache_keys[2], hit(json!({"user": {"someConstant": "123"}}), 5000));

    let FetchPhaseResult::PartialHit(execution) = fetch_phase.finish(no_subtypes()) else {
        panic!("We didn't hit everything so this should always be a partial");
    };

    let mut query_response = QueryResponse::default();
    let root_node = query_response.from_serde_value(json!({
        "user": {
            "name": "G",
            "email": "whatever",
            "nested": {"someThing": "hello"}
        }
    }));
    query_response.set_root_unchecked(root_node);

    let (response, _updates) = execution.handle_full_response(query_response, false);

    assert_eq!(
        response.headers.get("x-grafbase-cache"),
        Some(&HeaderValue::from_static("PARTIAL_HIT"))
    );

    let response_cache_control = response.headers.typed_get::<headers::CacheControl>().unwrap();
    assert_eq!(response_cache_control.max_age().unwrap().as_secs(), 130);
}

#[test]
fn test_miss_headers() {
    let registry = build_registry(SCHEMA);
    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();
    let fetch_phase = plan.start_fetch_phase(&auth(), &headers(), &variables());

    let FetchPhaseResult::PartialHit(execution) = fetch_phase.finish(no_subtypes()) else {
        panic!("We didn't hit everything so this should always be a partial");
    };

    let mut query_response = QueryResponse::default();
    let root_node = query_response.from_serde_value(json!({
        "user": {
            "name": "G",
            "email": "whatever",
            "someConstant": "123",
            "nested": {"someThing": "hello"}
        }
    }));
    query_response.set_root_unchecked(root_node);

    let (response, _updates) = execution.handle_full_response(query_response, false);

    assert_eq!(
        response.headers.get("x-grafbase-cache"),
        Some(&HeaderValue::from_static("MISS"))
    );

    let response_cache_control = response.headers.typed_get::<headers::CacheControl>().unwrap();
    assert_eq!(response_cache_control.max_age().unwrap().as_secs(), 120);
}

#[test]
fn test_query_that_hits_uncacheable_fields_should_have_no_cache_control_header() {
    let registry = build_registry(SCHEMA);
    let plan = partial_caching::build_plan("{ user { email uncacheable } }", None, &registry)
        .unwrap()
        .unwrap();
    let fetch_phase = plan.start_fetch_phase(&auth(), &headers(), &variables());

    let FetchPhaseResult::PartialHit(execution) = fetch_phase.finish(no_subtypes()) else {
        panic!("We didn't hit everything so this should always be a partial");
    };

    let mut query_response = QueryResponse::default();
    let root_node = query_response.from_serde_value(json!({
        "user": {
            "email": "whatever",
            "uncacheable": "Hello",
        }
    }));
    query_response.set_root_unchecked(root_node);

    let (response, _updates) = execution.handle_full_response(query_response, false);

    assert_eq!(
        response.headers.get("x-grafbase-cache"),
        Some(&HeaderValue::from_static("MISS"))
    );

    assert_eq!(response.headers.typed_get::<headers::CacheControl>(), None);
}

#[test]
fn test_streaming_handles_empty_deferred_objects() {
    let registry = build_registry(SCHEMA);
    const QUERY: &str = r#"query { user { ... @defer { name email someConstant nested }}}"#;

    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();
    let fetch_phase = plan.start_fetch_phase(&auth(), &headers(), &variables());

    let FetchPhaseResult::PartialHit(execution) = fetch_phase.finish(no_subtypes()) else {
        panic!("We didn't hit everything so this should always be a partial");
    };

    let mut execution = execution.streaming();

    let response = execution.record_initial_response(json!({"user": {}}).into(), false);

    assert_json_snapshot!(response, @r###"
    {
      "user": {}
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

fn hit(value: serde_json::Value, time_till_miss: u64) -> Entry<serde_json::Value> {
    Entry::Hit(value, Duration::from_secs(time_till_miss))
}

#![allow(unused_crate_dependencies, clippy::panic)]

use common_types::auth::ExecutionAuth;
use http::HeaderMap;
use insta::{assert_json_snapshot, assert_snapshot};
use partial_caching::FetchPhaseResult;
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
        uncached: String
    }
"#;

const QUERY: &str = r#"query SomeName { user { name email someConstant uncached } }"#;

#[test]
fn cache_partitions_need_cache_fetched() {
    let registry = build_registry(SCHEMA);
    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();
    let fetch_phase = plan.start_fetch_phase(&auth(), &headers(), &variables());

    let cache_keys = fetch_phase
        .cache_keys()
        .iter()
        .map(|key| key.to_string())
        .collect::<Vec<_>>();

    assert_json_snapshot!(cache_keys, @r###"
    [
      "e89684018782de6720bd7d3788879b9f6edfd38ac310798298e0ade57bb35120",
      "710ae8fca46776cd4dbec55725b7a92453bd9ef2e82735807329ab86aa14a900",
      "e0221b10004e1c356f27a8983a5a26a6b41c7afe99d7283ebba5ff2c834839c8"
    ]
    "###);
}

#[test]
fn correct_query_when_some_miss_some_hits() {
    let registry = build_registry(SCHEMA);
    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();
    let mut fetch_phase = plan.start_fetch_phase(&auth(), &headers(), &variables());

    let cache_keys = fetch_phase.cache_keys();
    fetch_phase.record_cache_entry(
        &cache_keys[0],
        runtime::cache::Entry::Hit(json!({"user": {"name": "Jane"}})),
    );
    fetch_phase.record_cache_entry(&cache_keys[1], runtime::cache::Entry::Miss);

    let FetchPhaseResult::PartialHit(execution) = fetch_phase.finish() else {
        panic!("We didn't hit everything so this should always be a partial");
    };

    assert_snapshot!(execution.query(), @r###"
    query SomeName {
      user {
        email
        someConstant
        uncached
      }
    }
    "###)
}

#[test]
fn nocache_fields_are_always_in_query() {
    let registry = build_registry(SCHEMA);
    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();
    let mut fetch_phase = plan.start_fetch_phase(&auth(), &headers(), &variables());

    let cache_keys = fetch_phase.cache_keys();
    for key in &cache_keys {
        fetch_phase.record_cache_entry(key, runtime::cache::Entry::Hit(json!("whatever")));
    }

    let FetchPhaseResult::PartialHit(execution) = fetch_phase.finish() else {
        panic!("We have no cache fields hit everything so this should always be a partial");
    };

    assert_snapshot!(execution.query(), @r###"
    query SomeName {
      user {
        uncached
      }
    }
    "###)
}

#[test]
fn test_complete_cache_hits() {
    let registry = build_registry(SCHEMA);
    let plan = partial_caching::build_plan("query { user { name } }", None, &registry)
        .unwrap()
        .unwrap();
    let mut fetch_phase = plan.start_fetch_phase(&auth(), &headers(), &variables());

    let cache_keys = fetch_phase.cache_keys();
    assert_eq!(cache_keys.len(), 1);
    fetch_phase.record_cache_entry(&cache_keys[0], runtime::cache::Entry::Hit(json!("whatever")));

    let FetchPhaseResult::CompleteHit(_) = fetch_phase.finish() else {
        panic!("We hit all the cached fields so should have a complete hit");
    };
}

#[test]
fn query_name_does_not_factor_into_cache_key() {
    let registry = build_registry(SCHEMA);
    let plan = partial_caching::build_plan("query Hello { user { name } }", None, &registry)
        .unwrap()
        .unwrap();
    let fetch_phase = plan.start_fetch_phase(&auth(), &headers(), &variables());

    let cache_keys = fetch_phase.cache_keys();
    let cache_key_one = cache_keys.first().unwrap();

    let plan = partial_caching::build_plan("query { user { name } }", None, &registry)
        .unwrap()
        .unwrap();
    let fetch_phase = plan.start_fetch_phase(&auth(), &headers(), &variables());

    let cache_keys = fetch_phase.cache_keys();
    let cache_key_two = cache_keys.first().unwrap();

    assert_eq!(cache_key_one.to_string(), cache_key_two.to_string())
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

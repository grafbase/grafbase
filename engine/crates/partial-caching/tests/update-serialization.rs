#![allow(unused_crate_dependencies, clippy::panic)]

use common_types::auth::ExecutionAuth;
use graph_entities::QueryResponse;
use headers::HeaderMapExt;
use http::HeaderMap;
use partial_caching::{type_relationships::no_subtypes, FetchPhaseResult};
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
        someThing: String @cache(maxAge: 140)
        uncached: String
    }
"#;

const QUERY: &str = r#"query { user { name email someConstant nested { uncached someThing }}}"#;

#[test]
fn test_serializing_all_updates() {
    let registry = build_registry(SCHEMA);
    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();
    let fetch_phase = plan.start_fetch_phase(&auth(), &headers(), &variables());

    let FetchPhaseResult::PartialHit(execution) = fetch_phase.finish(no_subtypes()) else {
        panic!("We didn't hit everything so this should always be a partial");
    };

    let mut executor_response = QueryResponse::default();
    let root_node = executor_response.from_serde_value(json!({
        "user": {
            "name": "Jane",
            "email": "whatever",
            "someConstant": "123",
            "nested": {
                "someThing": "whatever",
                "uncached": "Blah de blah"
            }
        }

    }));
    executor_response.set_root_unchecked(root_node);

    let (actual_response, update_phase) = execution.handle_full_response(executor_response.clone(), false);

    assert_eq!(
        actual_response.body.to_json_value().unwrap(),
        executor_response.to_json_value().unwrap()
    );

    let Some(update_phase) = update_phase else {
        panic!("we should definitely have some updates here");
    };
    let updates = update_phase.updates().collect::<Vec<_>>();

    assert_eq!(updates.len(), 3);

    // This should have name & nested.someThing in it
    insta::assert_json_snapshot!(updates[0], @r###"
    {
      "user": {
        "name": "Jane",
        "nested": {
          "someThing": "whatever"
        }
      }
    }
    "###);

    // This should have email
    insta::assert_json_snapshot!(updates[1], @r###"
    {
      "user": {
        "email": "whatever"
      }
    }
    "###);

    // This should have someConstant
    insta::assert_json_snapshot!(updates[2], @r###"
    {
      "user": {
        "someConstant": "123"
      }
    }
    "###);
}

#[test]
fn no_updates_if_errors() {
    let registry = build_registry(SCHEMA);
    let plan = partial_caching::build_plan("{ user { name } }", None, &registry)
        .unwrap()
        .unwrap();
    let fetch_phase = plan.start_fetch_phase(&auth(), &headers(), &variables());

    let FetchPhaseResult::PartialHit(execution) = fetch_phase.finish(no_subtypes()) else {
        panic!("We didn't hit everything so this should always be a partial");
    };

    let mut executor_response = QueryResponse::default();
    let root_node = executor_response.from_serde_value(json!({"user": {"name": "Jane"}}));
    executor_response.set_root_unchecked(root_node);

    let (actual_response, update_phase) = execution.handle_full_response(executor_response.clone(), true);

    assert_eq!(
        actual_response.body.to_json_value().unwrap(),
        executor_response.to_json_value().unwrap()
    );

    assert!(update_phase.is_none())
}

#[test]
fn no_updates_if_no_store_header_provided() {
    let registry = build_registry(SCHEMA);
    let plan = partial_caching::build_plan("{ user { name } }", None, &registry)
        .unwrap()
        .unwrap();

    let mut headers = http::HeaderMap::new();
    headers.typed_insert(headers::CacheControl::new().with_no_store());

    let fetch_phase = plan.start_fetch_phase(&auth(), &headers, &variables());

    let FetchPhaseResult::PartialHit(execution) = fetch_phase.finish(no_subtypes()) else {
        panic!("We didn't hit everything so this should always be a partial");
    };

    let mut executor_response = QueryResponse::default();
    let root_node = executor_response.from_serde_value(json!({"user": {"name": "Jane"}}));
    executor_response.set_root_unchecked(root_node);

    let (_, update_phase) = execution.handle_full_response(executor_response.clone(), true);

    assert!(update_phase.is_none())
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

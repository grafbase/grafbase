//! Unit tests for the output module.
//!
//! These might get deleted in favour of integration tests at some point, but
//! this module isn't hooked up at all just now so they're kinda useful

use graph_entities::QueryResponse;
use internment::ArcIntern;
use query_path::QueryPathSegment;
use serde_json::json;

use crate::output::engine_response::InitialOutput;

use super::shapes::build_output_shapes;

macro_rules! query_response {
    ($($json:tt)+) => {
        self::query_response(serde_json::json!($($json)+))
    }
}

#[test]
fn test_initial_response_handling() {
    const QUERY: &str = r#"{ user { name email someConstant nested { someThing } } }"#;

    let document = cynic_parser::parse_executable_document(QUERY).unwrap();
    let operation = document.operations().next().unwrap();

    let shapes = build_output_shapes(operation);
    let root_shape = shapes.root();

    let query_response = query_response!({
        "user": {
            "name": "G",
            "email": "whatever",
            "someConstant": "123",
            "nested": [{"someThing": "hello"}, {"someThing": "goodbye"}]
        }
    });

    let output = InitialOutput::new(query_response, root_shape);

    insta::assert_json_snapshot!(output.store.serialize_all(&shapes, serde_json::value::Serializer).unwrap(), @r###"
    {
      "user": {
        "name": "G",
        "email": "whatever",
        "someConstant": "123",
        "nested": [
          {
            "someThing": "hello"
          },
          {
            "someThing": "goodbye"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_cache_merging() {
    const QUERY: &str = r#"{ user { name email cacheThing nested { someThing cacheThing } } }"#;

    let document = cynic_parser::parse_executable_document(QUERY).unwrap();
    let operation = document.operations().next().unwrap();

    let shapes = build_output_shapes(operation);
    let root_shape = shapes.root();

    let query_response = query_response!({
        "user": {
            "name": "G",
            "email": "whatever",
            "nested": [{"someThing": "hello"}, {"someThing": "goodbye"}]
        }
    });

    let InitialOutput {
        mut store,
        active_defers,
    } = InitialOutput::new(query_response, root_shape);

    store.merge_cache_entry(
        &mut json!({
            "user": {
                "cacheThing": "I come from the cache",
                "nested": [
                    {"cacheThing": "I also come from the cache"},
                    {"cacheThing": "you better believe I am cached"}
                ]
            }
        }),
        &shapes,
        Some(active_defers),
    );

    insta::assert_json_snapshot!(store.serialize_all(&shapes, serde_json::value::Serializer).unwrap(), @r###"
    {
      "user": {
        "name": "G",
        "email": "whatever",
        "cacheThing": "I come from the cache",
        "nested": [
          {
            "someThing": "hello",
            "cacheThing": "I also come from the cache"
          },
          {
            "someThing": "goodbye",
            "cacheThing": "you better believe I am cached"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_cache_merging_with_defer() {
    const QUERY: &str = r#"{
        user {
            name
            email
            cacheThing
            ... @defer(label: "foo") {
                nested {
                    cacheThing
                }
            }
        }
    }"#;

    let document = cynic_parser::parse_executable_document(QUERY).unwrap();
    let operation = document.operations().next().unwrap();

    let shapes = build_output_shapes(operation);
    let root_shape = shapes.root();

    let query_response = query_response!({
        "user": {
            "name": "G",
            "email": "whatever",
        }
    });

    let InitialOutput {
        mut store,
        active_defers,
    } = InitialOutput::new(query_response, root_shape);

    store.merge_cache_entry(
        &mut json!({
            "user": {
                "cacheThing": "I come from the cache",
                "nested": [
                    {"cacheThing": "I also come from the cache"},
                    {"cacheThing": "you better believe I am cached"}
                ]
            }
        }),
        &shapes,
        Some(active_defers),
    );

    // Everything in the cache was part of the defer so we should only
    // have the name & email here
    insta::assert_json_snapshot!(store.serialize_all(&shapes, serde_json::value::Serializer).unwrap(), @r###"
    {
      "user": {
        "name": "G",
        "email": "whatever",
        "cacheThing": "I come from the cache"
      }
    }
    "###);
}

#[test]
fn test_cache_merging_when_defer_ignored() {
    // Servers don't have to defer fields that are behind a @defer.
    // This tests handling of that case.
    const QUERY: &str = r#"{
        user {
            name
            cacheThing
            ... @defer(label: "foo") {
                email
                nested {
                    cacheThing
                }
            }
        }
    }"#;

    let document = cynic_parser::parse_executable_document(QUERY).unwrap();
    let operation = document.operations().next().unwrap();

    let shapes = build_output_shapes(operation);
    let root_shape = shapes.root();

    let query_response = query_response!({
        "user": {
            "name": "G",
            "email": "whatever",
        }
    });

    let InitialOutput {
        mut store,
        active_defers,
    } = InitialOutput::new(query_response, root_shape);

    assert!(active_defers.contains("foo"));

    store.merge_cache_entry(
        &mut json!({
            "user": {
                "cacheThing": "I come from the cache",
                "nested": [
                    {"cacheThing": "I also come from the cache"},
                    {"cacheThing": "you better believe I am cached"}
                ]
            }
        }),
        &shapes,
        Some(active_defers),
    );

    // Everything in the cache was part of the defer so we should only
    // have the name & email here
    insta::assert_json_snapshot!(store.serialize_all(&shapes, serde_json::value::Serializer).unwrap(), @r###"
    {
      "user": {
        "name": "G",
        "cacheThing": "I come from the cache",
        "email": "whatever",
        "nested": [
          {
            "cacheThing": "I also come from the cache"
          },
          {
            "cacheThing": "you better believe I am cached"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_incremental_response_merging() {
    const QUERY: &str = r#"{
        user {
            name
            email
            cacheThing
            ... @defer(label: "foo") {
                nonCached
                nested {
                    nonCached
                    cacheThing
                }
            }
        }
    }"#;

    let document = cynic_parser::parse_executable_document(QUERY).unwrap();
    let operation = document.operations().next().unwrap();

    let shapes = build_output_shapes(operation);
    let root_shape = shapes.root();

    let query_response = query_response!({
        "user": {
            "name": "G",
            "email": "whatever",
        }
    });

    let InitialOutput { mut store, .. } = InitialOutput::new(query_response, root_shape);

    store.merge_cache_entry(
        &mut json!({
            "user": {
                "cacheThing": "I come from the cache",
                "nested": [
                    {"cacheThing": "I also come from the cache"},
                    {"cacheThing": "you better believe I am cached"}
                ]
            }
        }),
        &shapes,
        None,
    );

    store.merge_incremental_payload(
        &[&QueryPathSegment::Field(ArcIntern::new("user".into()))],
        query_response!({
            "nonCached": "I was not cached",
            "nested": [
                {"nonCached": "nor was I"},
                {"nonCached": "nor I"},
            ]
        }),
        &shapes,
    );

    // Everything in the cache was part of the defer so we should only
    // have the name & email here
    insta::assert_json_snapshot!(store.serialize_all(&shapes, serde_json::value::Serializer).unwrap(), @r###"
    {
      "user": {
        "name": "G",
        "email": "whatever",
        "cacheThing": "I come from the cache",
        "nonCached": "I was not cached",
        "nested": [
          {
            "nonCached": "nor was I",
            "cacheThing": "I also come from the cache"
          },
          {
            "nonCached": "nor I",
            "cacheThing": "you better believe I am cached"
          }
        ]
      }
    }
    "###);
}

fn build_registry(schema: &str) -> registry_for_cache::PartialCacheRegistry {
    registry_upgrade::convert_v1_to_partial_cache_registry(parser_sdl::parse_registry(schema).unwrap()).unwrap()
}

fn query_response(json: serde_json::Value) -> QueryResponse {
    let mut query_response = QueryResponse::default();
    let root_node = query_response.from_serde_value(json);
    query_response.set_root_unchecked(root_node);
    query_response
}

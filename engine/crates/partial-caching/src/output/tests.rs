//! Unit tests for the output module.
//!
//! These might get deleted in favour of integration tests at some point, but
//! this module isn't hooked up at all just now so they're kinda useful

use graph_entities::QueryResponse;
use serde_json::json;

use crate::output::engine_response::InitialOutput;

use super::shapes::build_output_shapes;

#[test]
fn test_initial_response_handling() {
    const QUERY: &str = r#"{ user { name email someConstant nested { someThing } } }"#;

    let document = cynic_parser::parse_executable_document(QUERY).unwrap();
    let operation = document.operations().next().unwrap();

    let shapes = build_output_shapes(operation);
    let root_shape = shapes.root();

    let mut query_response = QueryResponse::default();
    let root_node = query_response.from_serde_value(json!({
        "user": {
            "name": "G",
            "email": "whatever",
            "someConstant": "123",
            "nested": [{"someThing": "hello"}, {"someThing": "goodbye"}]
        }
    }));
    query_response.set_root_unchecked(root_node);

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

    let mut query_response = QueryResponse::default();
    let root_node = query_response.from_serde_value(json!({
        "user": {
            "name": "G",
            "email": "whatever",
            "nested": [{"someThing": "hello"}, {"someThing": "goodbye"}]
        }
    }));
    query_response.set_root_unchecked(root_node);

    let mut output = InitialOutput::new(query_response, root_shape);

    output.merge_cache_entry(
        json!({
            "user": {
                "cacheThing": "I come from the cache",
                "nested": [
                    {"cacheThing": "I also come from the cache"},
                    {"cacheThing": "you better believe I am cached"}
                ]
            }
        }),
        &shapes,
    );

    insta::assert_json_snapshot!(output.store.serialize_all(&shapes, serde_json::value::Serializer).unwrap(), @r###"
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

    let mut query_response = QueryResponse::default();
    let root_node = query_response.from_serde_value(json!({
        "user": {
            "name": "G",
            "email": "whatever",
        }
    }));
    query_response.set_root_unchecked(root_node);

    let mut output = InitialOutput::new(query_response, root_shape);

    output.merge_cache_entry(
        json!({
            "user": {
                "cacheThing": "I come from the cache",
                "nested": [
                    {"cacheThing": "I also come from the cache"},
                    {"cacheThing": "you better believe I am cached"}
                ]
            }
        }),
        &shapes,
    );

    // Everything in the cache was part of the defer so we should only
    // have the name & email here
    insta::assert_json_snapshot!(output.store.serialize_all(&shapes, serde_json::value::Serializer).unwrap(), @r###"
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

    let mut query_response = QueryResponse::default();
    let root_node = query_response.from_serde_value(json!({
        "user": {
            "name": "G",
            "email": "whatever",
        }
    }));
    query_response.set_root_unchecked(root_node);

    let mut output = InitialOutput::new(query_response, root_shape);

    assert!(output.active_defers.contains("foo"));

    output.merge_cache_entry(
        json!({
            "user": {
                "cacheThing": "I come from the cache",
                "nested": [
                    {"cacheThing": "I also come from the cache"},
                    {"cacheThing": "you better believe I am cached"}
                ]
            }
        }),
        &shapes,
    );

    // Everything in the cache was part of the defer so we should only
    // have the name & email here
    insta::assert_json_snapshot!(output.store.serialize_all(&shapes, serde_json::value::Serializer).unwrap(), @r###"
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

fn build_registry(schema: &str) -> registry_for_cache::PartialCacheRegistry {
    registry_upgrade::convert_v1_to_partial_cache_registry(parser_sdl::parse_registry(schema).unwrap()).unwrap()
}

//! Unit tests for the output module.
//!
//! These might get deleted in favour of integration tests at some point, but
//! this module isn't hooked up at all just now so they're kinda useful

use graph_entities::QueryResponse;
use serde_json::json;

use super::{shapes::build_output_shapes, OutputStore};

// For ease of testing none of these things are cacheable
const SCHEMA: &str = r#"
    type Query {
        user: User @resolver(name: "whatever")
    }

    type User {
        name: String
        email: String
        someConstant: String
        nested: [Nested]
    }

    type Nested {
        someThing: String
    }
"#;

const QUERY: &str = r#"{ user { name email someConstant nested { someThing } } }"#;

#[test]
fn test_initial_response_handling() {
    // Currently don't need this, but I am assuming I will later so just keeping it around
    let _registry = build_registry(SCHEMA);

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

    let output = OutputStore::new(query_response, root_shape);

    insta::assert_json_snapshot!(output.serialize_all(&shapes, serde_json::value::Serializer).unwrap(), @r###"
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

fn build_registry(schema: &str) -> registry_for_cache::PartialCacheRegistry {
    registry_upgrade::convert_v1_to_partial_cache_registry(parser_sdl::parse_registry(schema).unwrap()).unwrap()
}

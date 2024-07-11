//! Unit tests for the output module.
//!
//! These might get deleted in favour of integration tests at some point, but
//! this module isn't hooked up at all just now so they're kinda useful

use std::collections::HashSet;

use graph_entities::CompactValue;
use serde_json::json;

use crate::{build_plan, output::handle_initial_response, type_relationships::NoSubtypes};

use super::shapes::build_output_shapes;

macro_rules! compact_value {
    ($($json:tt)+) => {
        self::compact_value(serde_json::json!($($json)+))
    }
}

#[test]
fn test_initial_response_handling() {
    const QUERY: &str = r#"{ user { name email someConstant nested { someThing } } }"#;

    let plan = build_plan(QUERY, None, &registry()).unwrap().unwrap();

    let shapes = build_output_shapes(&plan, &NoSubtypes);
    let root_shape = shapes.root();

    let compact_value = compact_value!({
        "user": {
            "name": "G",
            "email": "whatever",
            "someConstant": "123",
            "nested": [{"someThing": "hello"}, {"someThing": "goodbye"}]
        }
    });

    let (store, _) = handle_initial_response(compact_value, &shapes, root_shape, &NoSubtypes);

    insta::assert_json_snapshot!(store.serialize_all(&shapes, serde_json::value::Serializer).unwrap(), @r###"
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

fn registry() -> registry_for_cache::PartialCacheRegistry {
    registry_upgrade::convert_v1_to_partial_cache_registry(
        parser_sdl::parse_registry("type Query { field: String @resolver(name: \"whateves\") }").unwrap(),
    )
    .unwrap()
}

#[test]
fn test_cache_merging() {
    const QUERY: &str = r#"{ user { name email cacheThing nested { someThing cacheThing } } }"#;

    let plan = build_plan(QUERY, None, &registry()).unwrap().unwrap();

    let shapes = build_output_shapes(&plan, &NoSubtypes);
    let root_shape = shapes.root();

    let compact_value = compact_value!({
        "user": {
            "name": "G",
            "email": "whatever",
            "nested": [{"someThing": "hello"}, {"someThing": "goodbye"}]
        }
    });

    let (mut store, active_defers) = handle_initial_response(compact_value, &shapes, root_shape, &NoSubtypes);

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
        &active_defers,
        &NoSubtypes,
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

    let plan = build_plan(QUERY, None, &registry()).unwrap().unwrap();

    let shapes = build_output_shapes(&plan, &NoSubtypes);
    let root_shape = shapes.root();

    let compact_value = compact_value!({
        "user": {
            "name": "G",
            "email": "whatever",
        }
    });

    let (mut store, active_defers) = handle_initial_response(compact_value, &shapes, root_shape, &NoSubtypes);

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
        &active_defers,
        &NoSubtypes,
    );

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

    let plan = build_plan(QUERY, None, &registry()).unwrap().unwrap();
    let defer_id = plan.defers().next().unwrap().id;

    let shapes = build_output_shapes(&plan, &NoSubtypes);
    let root_shape = shapes.root();

    let compact_value = compact_value!({
        "user": {
            "name": "G",
            "email": "whatever",
        }
    });

    let (mut store, active_defers) = handle_initial_response(compact_value, &shapes, root_shape, &NoSubtypes);

    assert!(active_defers.contains(&defer_id));

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
        &active_defers,
        &NoSubtypes,
    );

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
                    nestedCacheThing
                }
            }
        }
    }"#;

    let plan = build_plan(QUERY, None, &registry()).unwrap().unwrap();
    let defer_id = plan.defers().next().unwrap().id;

    let shapes = build_output_shapes(&plan, &NoSubtypes);
    let root_shape = shapes.root();

    let compact_value = compact_value!({
        "user": {
            "name": "G",
            "email": "whatever",
        }
    });

    let (mut store, active_defers) = handle_initial_response(compact_value, &shapes, root_shape, &NoSubtypes);

    let mut cache_entry = json!({
        "user": {
            "cacheThing": "I come from the cache",
            "nested": [
                {"nestedCacheThing": "I also come from the cache"},
                {"nestedCacheThing": "you better believe I am cached"}
            ]
        }
    });

    store.merge_cache_entry(&mut cache_entry, &shapes, &active_defers, &NoSubtypes);

    store.merge_specific_defer_from_cache_entry(&mut cache_entry, &shapes, defer_id, &HashSet::new(), &NoSubtypes);

    let crate::output::Value::Object(object) = store.reader(&shapes).unwrap().field("user").unwrap() else {
        unreachable!()
    };
    let user_object_id = object.id;

    store.merge_incremental_payload(
        user_object_id,
        compact_value!({
            "nonCached": "I was not cached",
            "nested": [
                {"nonCached": "nor was I"},
                {"nonCached": "nor I"},
            ]
        }),
        &shapes,
        &NoSubtypes,
    );

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
            "nestedCacheThing": "I also come from the cache"
          },
          {
            "nonCached": "nor I",
            "nestedCacheThing": "you better believe I am cached"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_nested_defers() {
    const QUERY: &str = r#"{
        user {
            name
            email
            cacheThing
            ... @defer(label: "foo") {
                nonCached
                ... @defer(label: "bar") {
                    nested {
                        nonCached
                        nestedCacheThing
                    }
                }
            }
        }
    }"#;

    let plan = build_plan(QUERY, None, &registry()).unwrap().unwrap();
    let mut defers = plan.defers();
    let first_defer_id = defers.next().unwrap().id;
    let second_defer_id = defers.next().unwrap().id;

    let shapes = build_output_shapes(&plan, &NoSubtypes);
    let root_shape = shapes.root();

    let compact_value = compact_value!({
        "user": {
            "name": "G",
            "email": "whatever",
        }
    });

    let (mut store, active_defers) = handle_initial_response(compact_value, &shapes, root_shape, &NoSubtypes);

    let mut cache_entry = json!({
        "user": {
            "cacheThing": "I come from the cache",
            "nested": [
                {"nestedCacheThing": "I also come from the cache"},
                {"nestedCacheThing": "you better believe I am cached"}
            ]
        }
    });

    store.merge_cache_entry(&mut cache_entry, &shapes, &active_defers, &NoSubtypes);

    let crate::output::Value::Object(object) = store.reader(&shapes).unwrap().field("user").unwrap() else {
        unreachable!()
    };
    let user_object_id = object.id;

    let active_defers = store.merge_incremental_payload(
        user_object_id,
        compact_value!({"nonCached": "I was not cached"}),
        &shapes,
        &NoSubtypes,
    );
    store.merge_specific_defer_from_cache_entry(&mut cache_entry, &shapes, first_defer_id, &active_defers, &NoSubtypes);

    insta::assert_json_snapshot!(store.serialize_all(&shapes, serde_json::value::Serializer).unwrap(), @r###"
    {
      "user": {
        "name": "G",
        "email": "whatever",
        "cacheThing": "I come from the cache",
        "nonCached": "I was not cached"
      }
    }
    "###);

    let active_defers = store.merge_incremental_payload(
        user_object_id,
        compact_value!({
            "nested": [
                {"nonCached": "nor was I"},
                {"nonCached": "nor I"},
            ]
        }),
        &shapes,
        &NoSubtypes,
    );

    store.merge_specific_defer_from_cache_entry(
        &mut cache_entry,
        &shapes,
        second_defer_id,
        &active_defers,
        &NoSubtypes,
    );

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
            "nestedCacheThing": "I also come from the cache"
          },
          {
            "nonCached": "nor I",
            "nestedCacheThing": "you better believe I am cached"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_nested_defers_when_defer_ignored() {
    const QUERY: &str = r#"{
        user {
            name
            email
            cacheThing
            ... @defer(label: "foo") {
                nonCached
                ... @defer(label: "bar") {
                    nested {
                        nonCached
                        nestedCacheThing
                    }
                }
            }
        }
    }"#;

    let plan = build_plan(QUERY, None, &registry()).unwrap().unwrap();
    let mut defers = plan.defers();
    let first_defer_id = defers.next().unwrap().id;

    let shapes = build_output_shapes(&plan, &NoSubtypes);
    let root_shape = shapes.root();

    let compact_value = compact_value!({
        "user": {
            "name": "G",
            "email": "whatever",
        }
    });

    let (mut store, active_defers) = handle_initial_response(compact_value, &shapes, root_shape, &NoSubtypes);

    let mut cache_entry = json!({
        "user": {
            "cacheThing": "I come from the cache",
            "nested": [
                {"nestedCacheThing": "I also come from the cache"},
                {"nestedCacheThing": "you better believe I am cached"}
            ]
        }
    });

    store.merge_cache_entry(&mut cache_entry, &shapes, &active_defers, &NoSubtypes);

    let crate::output::Value::Object(object) = store.reader(&shapes).unwrap().field("user").unwrap() else {
        unreachable!()
    };
    let user_object_id = object.id;

    let active_defers = store.merge_incremental_payload(
        user_object_id,
        compact_value!({
            "nonCached": "I was not cached",
            "nested": [
                {"nonCached": "nor was I"},
                {"nonCached": "nor I"},
            ]
        }),
        &shapes,
        &NoSubtypes,
    );
    store.merge_specific_defer_from_cache_entry(&mut cache_entry, &shapes, first_defer_id, &active_defers, &NoSubtypes);

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
            "nestedCacheThing": "I also come from the cache"
          },
          {
            "nonCached": "nor I",
            "nestedCacheThing": "you better believe I am cached"
          }
        ]
      }
    }
    "###);
}

fn compact_value(json: serde_json::Value) -> CompactValue {
    json.into()
}

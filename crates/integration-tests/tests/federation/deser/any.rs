use serde_json::json;

use super::run;

const REQUIRED_ANY_SCHEMA: &str = r#"
scalar Any

type Query {
    user: User @join__field(graph: A)
    dummy: String @join__field(graph: A)
}
type User {
    something: Any! @join__field(graph: A)
    valid: String @join__field(graph: A)
}
"#;

const NULLABLE_ANY_SCHEMA: &str = r#"
scalar Any

type Query {
    user: User @join__field(graph: A)
    dummy: String @join__field(graph: A)
}
type User {
    something: Any @join__field(graph: A)
    valid: String @join__field(graph: A)
}
"#;

const QUERY: &str = r#"
{
    user {
        something
        valid
    }
    dummy
}"#;

#[test]
fn expected_required_any_got_string() {
    let response = run(
        REQUIRED_ANY_SCHEMA,
        QUERY,
        json!({"data": {"user": {"something": "Bob", "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "something": "Bob",
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_any_got_string() {
    let response = run(
        NULLABLE_ANY_SCHEMA,
        QUERY,
        json!({"data": {"user": {"something": "Alice", "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "something": "Alice",
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_any_got_null() {
    let response = run(
        REQUIRED_ANY_SCHEMA,
        QUERY,
        json!({"data": {"user": {"something": null, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "something": null,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_any_got_bool() {
    let response = run(
        REQUIRED_ANY_SCHEMA,
        QUERY,
        json!({"data": {"user": {"something": false, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "something": false,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_any_got_bool() {
    let response = run(
        NULLABLE_ANY_SCHEMA,
        QUERY,
        json!({"data": {"user": {"something": false, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "something": false,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_any_got_int() {
    let response = run(
        REQUIRED_ANY_SCHEMA,
        QUERY,
        json!({"data": {"user": {"something": 1, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "something": 1,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_any_got_int() {
    let response = run(
        NULLABLE_ANY_SCHEMA,
        QUERY,
        json!({"data": {"user": {"something": 1, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "something": 1,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_any_got_float() {
    let response = run(
        REQUIRED_ANY_SCHEMA,
        QUERY,
        json!({"data": {"user": {"something": 1.24, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "something": 1.24,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_any_got_float() {
    let response = run(
        NULLABLE_ANY_SCHEMA,
        QUERY,
        json!({"data": {"user": {"something": 1.24, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "something": 1.24,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_any_got_list() {
    let response = run(
        REQUIRED_ANY_SCHEMA,
        QUERY,
        json!({"data": {"user": {"something": ["test", 1.0], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "something": [
            "test",
            1.0
          ],
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_any_got_list() {
    let response = run(
        NULLABLE_ANY_SCHEMA,
        QUERY,
        json!({"data": {"user": {"something": ["test", 1.0], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "something": [
            "test",
            1.0
          ],
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_any_got_object() {
    let response = run(
        REQUIRED_ANY_SCHEMA,
        QUERY,
        json!({"data": {"user": {"something": {"k1": 1.0, "k2": "test"}, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "something": {
            "k1": 1.0,
            "k2": "test"
          },
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_any_got_object() {
    let response = run(
        NULLABLE_ANY_SCHEMA,
        QUERY,
        json!({"data": {"user": {"something": {"k1": 1.0, "k2": "test"}, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "something": {
            "k1": 1.0,
            "k2": "test"
          },
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

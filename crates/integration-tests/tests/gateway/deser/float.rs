use serde_json::json;

use super::run;

const REQUIRED_FLOAT_SCHEMA: &str = r#"
type Query {
    user: User @join__field(graph: A)
    dummy: String @join__field(graph: A)
}
type User {
    age: Float! @join__field(graph: A)
    valid: String @join__field(graph: A)
}
"#;

const NULLABLE_FLOAT_SCHEMA: &str = r#"
type Query {
    user: User @join__field(graph: A)
    dummy: String @join__field(graph: A)
}
type User {
    age: Float @join__field(graph: A)
    valid: String @join__field(graph: A)
}
"#;

const QUERY: &str = r#"
{
    user {
        age
        valid
    }
    dummy
}"#;

#[test]
fn expected_required_float_got_string() {
    let response = run(
        REQUIRED_FLOAT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"age": "Bob", "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": null,
        "dummy": "yes"
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 4,
              "column": 9
            }
          ],
          "path": [
            "user",
            "age"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn expected_nullable_float_got_string() {
    let response = run(
        NULLABLE_FLOAT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"age": "Alice", "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "age": null,
          "valid": "yes"
        },
        "dummy": "yes"
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 4,
              "column": 9
            }
          ],
          "path": [
            "user",
            "age"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn expected_nullable_float_got_null() {
    let response = run(
        NULLABLE_FLOAT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"age": null, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "age": null,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_float_got_null() {
    let response = run(
        REQUIRED_FLOAT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"age": null, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": null,
        "dummy": "yes"
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 4,
              "column": 9
            }
          ],
          "path": [
            "user",
            "age"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn expected_required_float_got_bool() {
    let response = run(
        REQUIRED_FLOAT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"age": false, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": null,
        "dummy": "yes"
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 4,
              "column": 9
            }
          ],
          "path": [
            "user",
            "age"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn expected_nullable_float_got_bool() {
    let response = run(
        NULLABLE_FLOAT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"age": false, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "age": null,
          "valid": "yes"
        },
        "dummy": "yes"
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 4,
              "column": 9
            }
          ],
          "path": [
            "user",
            "age"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn expected_required_float_got_int() {
    let response = run(
        REQUIRED_FLOAT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"age": 1, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "age": 1.0,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_float_got_int() {
    let response = run(
        NULLABLE_FLOAT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"age": 1, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "age": 1.0,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_float_got_big_int() {
    let response = run(
        REQUIRED_FLOAT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"age": i64::MAX, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "age": 9223372036854776000.0,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_float_got_big_int() {
    let response = run(
        NULLABLE_FLOAT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"age": i64::MAX, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "age": 9223372036854776000.0,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_float_got_float() {
    let response = run(
        REQUIRED_FLOAT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"age": 1.24, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "age": 1.24,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_float_got_float() {
    let response = run(
        NULLABLE_FLOAT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"age": 1.24, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "age": 1.24,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_float_got_list() {
    let response = run(
        REQUIRED_FLOAT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"age": [], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": null,
        "dummy": "yes"
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 4,
              "column": 9
            }
          ],
          "path": [
            "user",
            "age"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn expected_nullable_float_got_list() {
    let response = run(
        NULLABLE_FLOAT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"age": [], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "age": null,
          "valid": "yes"
        },
        "dummy": "yes"
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 4,
              "column": 9
            }
          ],
          "path": [
            "user",
            "age"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn expected_required_float_got_object() {
    let response = run(
        REQUIRED_FLOAT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"age": {}, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": null,
        "dummy": "yes"
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 4,
              "column": 9
            }
          ],
          "path": [
            "user",
            "age"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn expected_nullable_float_got_object() {
    let response = run(
        NULLABLE_FLOAT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"age": {}, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "age": null,
          "valid": "yes"
        },
        "dummy": "yes"
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 4,
              "column": 9
            }
          ],
          "path": [
            "user",
            "age"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

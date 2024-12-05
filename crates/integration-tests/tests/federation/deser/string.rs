use serde_json::json;

use super::run;

const REQUIRED_STRING_SCHEMA: &str = r#"
type Query {
    user: User @join__field(graph: A)
    dummy: String @join__field(graph: A)
}
type User {
    username: String! @join__field(graph: A)
    valid: String @join__field(graph: A)
}
"#;

const NULLABLE_STRING_SCHEMA: &str = r#"
type Query {
    user: User @join__field(graph: A)
    dummy: String @join__field(graph: A)
}
type User {
    username: String @join__field(graph: A)
    valid: String @join__field(graph: A)
}
"#;

const QUERY: &str = r#"
{
    user {
        username
        valid
    }
    dummy
}"#;

#[test]
fn expected_required_string_got_string() {
    let response = run(
        REQUIRED_STRING_SCHEMA,
        QUERY,
        json!({"data": {"user": {"username": "Bob", "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "username": "Bob",
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_string_got_string() {
    let response = run(
        NULLABLE_STRING_SCHEMA,
        QUERY,
        json!({"data": {"user": {"username": "Alice", "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "username": "Alice",
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_string_got_null() {
    let response = run(
        NULLABLE_STRING_SCHEMA,
        QUERY,
        json!({"data": {"user": {"username": null, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "username": null,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}
#[test]
fn expected_required_string_got_null() {
    let response = run(
        REQUIRED_STRING_SCHEMA,
        QUERY,
        json!({"data": {"user": {"username": null, "valid": "yes"}, "dummy": "yes"}}),
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
            "username"
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
fn expected_required_string_got_bool() {
    let response = run(
        REQUIRED_STRING_SCHEMA,
        QUERY,
        json!({"data": {"user": {"username": false, "valid": "yes"}, "dummy": "yes"}}),
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
            "username"
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
fn expected_nullable_string_got_bool() {
    let response = run(
        NULLABLE_STRING_SCHEMA,
        QUERY,
        json!({"data": {"user": {"username": false, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "username": null,
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
            "username"
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
fn expected_required_string_got_int() {
    let response = run(
        REQUIRED_STRING_SCHEMA,
        QUERY,
        json!({"data": {"user": {"username": 1, "valid": "yes"}, "dummy": "yes"}}),
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
            "username"
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
fn expected_nullable_string_got_int() {
    let response = run(
        NULLABLE_STRING_SCHEMA,
        QUERY,
        json!({"data": {"user": {"username": 1, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "username": null,
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
            "username"
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
fn expected_required_string_got_float() {
    let response = run(
        REQUIRED_STRING_SCHEMA,
        QUERY,
        json!({"data": {"user": {"username": 1.24, "valid": "yes"}, "dummy": "yes"}}),
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
            "username"
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
fn expected_nullable_string_got_float() {
    let response = run(
        NULLABLE_STRING_SCHEMA,
        QUERY,
        json!({"data": {"user": {"username": 1.24, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "username": null,
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
            "username"
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
fn expected_required_string_got_list() {
    let response = run(
        REQUIRED_STRING_SCHEMA,
        QUERY,
        json!({"data": {"user": {"username": [], "valid": "yes"}, "dummy": "yes"}}),
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
            "username"
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
fn expected_nullable_string_got_list() {
    let response = run(
        NULLABLE_STRING_SCHEMA,
        QUERY,
        json!({"data": {"user": {"username": [], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "username": null,
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
            "username"
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
fn expected_required_string_got_object() {
    let response = run(
        REQUIRED_STRING_SCHEMA,
        QUERY,
        json!({"data": {"user": {"username": {}, "valid": "yes"}, "dummy": "yes"}}),
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
            "username"
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
fn expected_nullable_string_got_object() {
    let response = run(
        NULLABLE_STRING_SCHEMA,
        QUERY,
        json!({"data": {"user": {"username": {}, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "username": null,
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
            "username"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

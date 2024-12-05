use serde_json::json;

use super::run;

const REQUIRED_LIST_SCHEMA: &str = r#"
type Query {
    user: User @join__field(graph: A)
    dummy: String @join__field(graph: A)
}
type User {
    friends: [String!]! @join__field(graph: A)
    valid: String @join__field(graph: A)
}
"#;

const NULLABLE_LIST_SCHEMA: &str = r#"
type Query {
    user: User @join__field(graph: A)
    dummy: String @join__field(graph: A)
}
type User {
    friends: [String] @join__field(graph: A)
    valid: String @join__field(graph: A)
}
"#;

const QUERY: &str = r#"
{
    user {
        friends
        valid
    }
    dummy
}"#;

#[test]
fn expected_required_list_got_string() {
    let response = run(
        REQUIRED_LIST_SCHEMA,
        QUERY,
        json!({"data": {"user": {"friends": "Bob", "valid": "yes"}, "dummy": "yes"}}),
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
            "friends"
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
fn expected_nullable_list_got_string() {
    let response = run(
        NULLABLE_LIST_SCHEMA,
        QUERY,
        json!({"data": {"user": {"friends": "Alice", "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "friends": null,
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
            "friends"
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
fn expected_nullable_list_got_null() {
    let response = run(
        NULLABLE_LIST_SCHEMA,
        QUERY,
        json!({"data": {"user": {"friends": null, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "friends": null,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}
#[test]
fn expected_required_list_got_null() {
    let response = run(
        REQUIRED_LIST_SCHEMA,
        QUERY,
        json!({"data": {"user": {"friends": null, "valid": "yes"}, "dummy": "yes"}}),
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
            "friends"
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
fn expected_required_list_got_bool() {
    let response = run(
        REQUIRED_LIST_SCHEMA,
        QUERY,
        json!({"data": {"user": {"friends": false, "valid": "yes"}, "dummy": "yes"}}),
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
            "friends"
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
fn expected_nullable_list_got_bool() {
    let response = run(
        NULLABLE_LIST_SCHEMA,
        QUERY,
        json!({"data": {"user": {"friends": false, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "friends": null,
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
            "friends"
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
fn expected_required_list_got_int() {
    let response = run(
        REQUIRED_LIST_SCHEMA,
        QUERY,
        json!({"data": {"user": {"friends": 1, "valid": "yes"}, "dummy": "yes"}}),
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
            "friends"
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
fn expected_nullable_list_got_int() {
    let response = run(
        NULLABLE_LIST_SCHEMA,
        QUERY,
        json!({"data": {"user": {"friends": 1, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "friends": null,
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
            "friends"
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
fn expected_required_list_got_float() {
    let response = run(
        REQUIRED_LIST_SCHEMA,
        QUERY,
        json!({"data": {"user": {"friends": 1.24, "valid": "yes"}, "dummy": "yes"}}),
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
            "friends"
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
fn expected_nullable_list_got_float() {
    let response = run(
        NULLABLE_LIST_SCHEMA,
        QUERY,
        json!({"data": {"user": {"friends": 1.24, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "friends": null,
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
            "friends"
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
fn expected_required_list_got_list() {
    let response = run(
        REQUIRED_LIST_SCHEMA,
        QUERY,
        json!({"data": {"user": {"friends": ["Alice", "Bob"], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "friends": [
            "Alice",
            "Bob"
          ],
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_list_got_list() {
    let response = run(
        NULLABLE_LIST_SCHEMA,
        QUERY,
        json!({"data": {"user": {"friends": ["Bob", "Alice"], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "friends": [
            "Bob",
            "Alice"
          ],
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_list_got_list_with_invalid_element() {
    let response = run(
        REQUIRED_LIST_SCHEMA,
        QUERY,
        json!({"data": {"user": {"friends": ["Alice", {}], "valid": "yes"}, "dummy": "yes"}}),
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
            "friends",
            1
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
fn expected_nullable_list_got_list_with_invalid_element() {
    let response = run(
        NULLABLE_LIST_SCHEMA,
        QUERY,
        json!({"data": {"user": {"friends": ["Bob", {}], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "friends": [
            "Bob",
            null
          ],
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
            "friends",
            1
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
fn expected_required_list_got_object() {
    let response = run(
        REQUIRED_LIST_SCHEMA,
        QUERY,
        json!({"data": {"user": {"friends": {}, "valid": "yes"}, "dummy": "yes"}}),
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
            "friends"
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
fn expected_nullable_list_got_object() {
    let response = run(
        NULLABLE_LIST_SCHEMA,
        QUERY,
        json!({"data": {"user": {"friends": {}, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "friends": null,
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
            "friends"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

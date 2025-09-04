use serde_json::json;

use super::super::run;

const SCHEMA: &str = r#"
type Query {
    user: User @join__field(graph: A)
    dummy: String @join__field(graph: A)
}
type User {
    ratings: [Float] @join__field(graph: A)
    valid: String @join__field(graph: A)
}
"#;

const QUERY: &str = r#"
{
    user {
        ratings
        valid
    }
    dummy
}"#;

#[test]
fn got_valid_list() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"ratings": [1.5, 2.0, 3.5], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "ratings": [
            1.5,
            2.0,
            3.5
          ],
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn got_empty_list() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"ratings": [], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "ratings": [],
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn got_null() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"ratings": null, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "ratings": null,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn got_string() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"ratings": "invalid", "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "ratings": null,
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
            "ratings"
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
fn got_float() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"ratings": 42.5, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "ratings": null,
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
            "ratings"
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
fn got_int() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"ratings": 42, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "ratings": null,
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
            "ratings"
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
fn got_bool() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"ratings": true, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "ratings": null,
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
            "ratings"
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
fn got_object() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"ratings": {}, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "ratings": null,
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
            "ratings"
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
fn got_list_with_null() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"ratings": [1.5, null, 3.5], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "ratings": [
            1.5,
            null,
            3.5
          ],
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn got_list_with_string() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"ratings": [1.5, "invalid", 3.5], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "ratings": [
            1.5,
            null,
            3.5
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
            "ratings",
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
fn got_list_with_bool() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"ratings": [1.5, false, 3.5], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "ratings": [
            1.5,
            null,
            3.5
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
            "ratings",
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
fn got_list_with_int() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"ratings": [1.5, 2, 3.5], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "ratings": [
            1.5,
            2.0,
            3.5
          ],
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn got_list_with_object() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"ratings": [1.5, {}, 3.5], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "ratings": [
            1.5,
            null,
            3.5
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
            "ratings",
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
fn got_list_with_list() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"ratings": [1.5, [], 3.5], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "ratings": [
            1.5,
            null,
            3.5
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
            "ratings",
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

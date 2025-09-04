use serde_json::json;

use super::super::run;

const SCHEMA: &str = r#"
type Query {
    user: User @join__field(graph: A)
    dummy: String @join__field(graph: A)
}
type User {
    scores: [Int!]! @join__field(graph: A)
    valid: String @join__field(graph: A)
}
"#;

const QUERY: &str = r#"
{
    user {
        scores
        valid
    }
    dummy
}"#;

#[test]
fn got_valid_list() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"scores": [1, 2, 3], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "scores": [
            1,
            2,
            3
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
        json!({"data": {"user": {"scores": [], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "scores": [],
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
        json!({"data": {"user": {"scores": null, "valid": "yes"}, "dummy": "yes"}}),
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
            "scores"
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
fn got_string() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"scores": "invalid", "valid": "yes"}, "dummy": "yes"}}),
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
            "scores"
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
        json!({"data": {"user": {"scores": 42, "valid": "yes"}, "dummy": "yes"}}),
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
            "scores"
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
        json!({"data": {"user": {"scores": true, "valid": "yes"}, "dummy": "yes"}}),
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
            "scores"
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
        json!({"data": {"user": {"scores": 42.5, "valid": "yes"}, "dummy": "yes"}}),
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
            "scores"
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
        json!({"data": {"user": {"scores": {}, "valid": "yes"}, "dummy": "yes"}}),
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
            "scores"
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
        json!({"data": {"user": {"scores": [1, null, 3], "valid": "yes"}, "dummy": "yes"}}),
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
            "scores",
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
fn got_list_with_string() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"scores": [1, "invalid", 3], "valid": "yes"}, "dummy": "yes"}}),
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
            "scores",
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
        json!({"data": {"user": {"scores": [1, false, 3], "valid": "yes"}, "dummy": "yes"}}),
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
            "scores",
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
fn got_list_with_float() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"scores": [1, 2.5, 3], "valid": "yes"}, "dummy": "yes"}}),
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
            "scores",
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
fn got_list_with_coercable_float() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"scores": [1, 2.0, 3], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "scores": [
            1,
            2,
            3
          ],
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn got_list_with_big_int() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"scores": [1, i64::MAX, 3], "valid": "yes"}, "dummy": "yes"}}),
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
            "scores",
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
fn got_list_with_object() {
    let response = run(
        SCHEMA,
        QUERY,
        json!({"data": {"user": {"scores": [1, {}, 3], "valid": "yes"}, "dummy": "yes"}}),
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
            "scores",
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
        json!({"data": {"user": {"scores": [1, [], 3], "valid": "yes"}, "dummy": "yes"}}),
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
            "scores",
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

use serde_json::json;

use super::run;

const REQUIRED_OBJECT_SCHEMA: &str = r#"
type Query @join__type(graph: A) {
    user: User
    dummy: String
}
type User @join__type(graph: A) {
    organization: Organaization!
    valid: String
}
type Organaization @join__type(graph: A) {
    name: String!
    plan: String
}
"#;

const NULLABLE_OBJECT_SCHEMA: &str = r#"
type Query @join__type(graph: A) {
    user: User @join__field(graph: A)
    dummy: String @join__field(graph: A)
}
type User @join__type(graph: A) {
    organization: Organaization
    valid: String
}
type Organaization @join__type(graph: A) {
    name: String!
    plan: String
}
"#;

const QUERY: &str = r#"
{
    user {
        organization {
            name
            plan
        }
        valid
    }
    dummy
}"#;

#[test]
fn expected_required_object_got_string() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": "Bob", "valid": "yes"}, "dummy": "yes"}}),
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
            "organization"
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
fn expected_nullable_object_got_string() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": "Alice", "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "organization": null,
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
            "organization"
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
fn expected_nullable_object_got_null() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": null, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "organization": null,
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}
#[test]
fn expected_required_object_got_null() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": null, "valid": "yes"}, "dummy": "yes"}}),
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
            "organization"
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
fn expected_required_object_got_bool() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": false, "valid": "yes"}, "dummy": "yes"}}),
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
            "organization"
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
fn expected_nullable_object_got_bool() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": false, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "organization": null,
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
            "organization"
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
fn expected_required_object_got_int() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": 1, "valid": "yes"}, "dummy": "yes"}}),
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
            "organization"
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
fn expected_nullable_object_got_int() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": 1, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "organization": null,
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
            "organization"
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
fn expected_required_object_got_float() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": 1.24, "valid": "yes"}, "dummy": "yes"}}),
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
            "organization"
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
fn expected_nullable_object_got_float() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": 1.24, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "organization": null,
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
            "organization"
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
fn expected_required_object_got_list() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": ["test", 1.0], "valid": "yes"}, "dummy": "yes"}}),
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
            "organization"
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
fn expected_nullable_object_got_list() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": ["test", 1.0], "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "organization": null,
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
            "organization"
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
fn expected_required_object_got_object() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": {"name": "Grafbase", "plan":"super-enterprise"}, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "organization": {
            "name": "Grafbase",
            "plan": "super-enterprise"
          },
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_object_got_object() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": {"name": "Grafbase", "plan":"super-enterprise"}, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "organization": {
            "name": "Grafbase",
            "plan": "super-enterprise"
          },
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_object_got_object_with_missing_nullable_field() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": {"name": "Grafbase"}, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "organization": {
            "name": "Grafbase",
            "plan": null
          },
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_object_got_object_with_missing_nullable_field() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": {"name": "Grafbase"}, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "organization": {
            "name": "Grafbase",
            "plan": null
          },
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_object_got_object_with_missing_required_field() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": {"plan": "enterprise"}, "valid": "yes"}, "dummy": "yes"}}),
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
              "line": 5,
              "column": 13
            }
          ],
          "path": [
            "user",
            "organization",
            "name"
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
fn expected_nullable_object_got_object_with_missing_required_field() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": {"plan": "enterprise"}, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "organization": null,
          "valid": "yes"
        },
        "dummy": "yes"
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 5,
              "column": 13
            }
          ],
          "path": [
            "user",
            "organization",
            "name"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

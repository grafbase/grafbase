use serde_json::json;

use super::run;

const REQUIRED_OBJECT_SCHEMA: &str = r#"
type Query @join__type(graph: A) {
    user: User
    dummy: String
}
type User @join__type(graph: A) {
    organization: Organization!
    valid: String
}
interface Organization @join__type(graph: A) {
    id: ID!
}
type BigCorp implements Organization @join__type(graph: A) @join__implements(graph: A, interface: "Organization") {
    id: ID!
    valid: String
}
type SmallBusiness implements Organization @join__type(graph: A) @join__implements(graph: A, interface: "Organization") {
    id: ID!
    valid: String
}
type Something implements Organization {
    id: ID!
}
"#;

const NULLABLE_OBJECT_SCHEMA: &str = r#"
type Query @join__type(graph: A) {
    user: User
    dummy: String
}
type User @join__type(graph: A) {
    organization: Organization
    valid: String
}
interface Organization @join__type(graph: A) {
    id: ID!
}
type BigCorp implements Organization @join__type(graph: A) @join__implements(graph: A, interface: "Organization") {
    id: ID!
    valid: String
}
type SmallBusiness implements Organization @join__type(graph: A) @join__implements(graph: A, interface: "Organization") {
    id: ID!
    valid: String
}
type Something implements Organization {
    id: ID!
}
"#;

const QUERY: &str = r#"
{
    user {
        organization {
            id
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
        json!({"data": {"user": {"organization": [], "valid": "yes"}, "dummy": "yes"}}),
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
        json!({"data": {"user": {"organization": [], "valid": "yes"}, "dummy": "yes"}}),
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
        json!({"data": {"user": {"organization": {"id": "1"}, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "organization": {
            "id": "1"
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
        REQUIRED_OBJECT_SCHEMA,
        QUERY,
        json!({"data": {"user": {"organization": {"id": "1"}, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "organization": {
            "id": "1"
          },
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_required_object_missing_typename_1() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        r#"
        {
            user {
                organization {
                    id
                    ... on BigCorp {
                        valid
                    }
                }
                valid
            }
            dummy
        }
        "#,
        json!({"data": {"user": {"organization": {"id": "1"}, "valid": "yes"}, "dummy": "yes"}}),
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
              "column": 17
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
fn expected_required_object_missing_typename_2() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        r#"
        {
            user {
                organization {
                    ... on BigCorp {
                       id
                    }
                }
                valid
            }
            dummy 
        }
        "#,
        json!({"data": {"user": {"organization": {"id": "1"}, "valid": "yes"}, "dummy": "yes"}}),
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
              "column": 17
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
fn expected_required_object_missing_typename_3() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        r#"
        {
            user {
                organization {
                    ... on BigCorp {
                       id
                    }
                    ... on SmallBusiness {
                       id
                    }
                }
                valid
            }
            dummy 
        }
        "#,
        json!({"data": {"user": {"organization": {"id": "1"}, "valid": "yes"}, "dummy": "yes"}}),
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
              "column": 17
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
fn expected_required_object_no_match() {
    let response = run(
        REQUIRED_OBJECT_SCHEMA,
        r#"
        {
            user {
                organization {
                    ... on BigCorp {
                       id
                    }
                }
                valid
            }
            dummy
        }
        "#,
        json!({"data": {"user": {"organization": {"__typename": "SmallBusiness", "id": "1"}, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "organization": {},
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

#[test]
fn expected_nullable_object_missing_typename_1() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        r#"
        {
            user {
                organization {
                    id
                    ... on BigCorp {
                        valid
                    }
                }
                valid
            }
            dummy
        }
        "#,
        json!({"data": {"user": {"organization": {"id": "1"}, "valid": "yes"}, "dummy": "yes"}}),
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
              "column": 17
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
fn expected_nullable_object_missing_typename_2() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        r#"
        {
            user {
                organization {
                    ... on BigCorp {
                       id
                    }
                }
                valid
            }
            dummy 
        }
        "#,
        json!({"data": {"user": {"organization": {"id": "1"}, "valid": "yes"}, "dummy": "yes"}}),
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
              "column": 17
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
fn expected_nullable_object_missing_typename_3() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        r#"
        {
            user {
                organization {
                    ... on BigCorp {
                       id
                    }
                    ... on SmallBusiness {
                       id
                    }
                }
                valid
            }
            dummy 
        }
        "#,
        json!({"data": {"user": {"organization": {"id": "1"}, "valid": "yes"}, "dummy": "yes"}}),
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
              "column": 17
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
fn expected_nullable_object_no_match() {
    let response = run(
        NULLABLE_OBJECT_SCHEMA,
        r#"
        {
            user {
                organization {
                    ... on BigCorp {
                       id
                    }
                }
                valid
            }
            dummy
        }
        "#,
        json!({"data": {"user": {"organization": {"__typename": "SmallBusiness", "id": "1"}, "valid": "yes"}, "dummy": "yes"}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "organization": {},
          "valid": "yes"
        },
        "dummy": "yes"
      }
    }
    "#);
}

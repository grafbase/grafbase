use integration_tests::{federation::DeterministicEngine, runtime};
use serde_json::json;

use crate::federation::introspection::{introspection_to_sdl, PATHFINDER_INTROSPECTION_QUERY};

const SCHEMA: &str = r###"
directive @join__field(graph: join__Graph, requires: String, provides: String) on FIELD_DEFINITION
directive @join__graph(name: String!, url: String!) on ENUM_VALUE

enum join__Graph {
  PRODUCTS @join__graph(name: "products", url: "http://127.0.0.1:46697/some-random-url")
}

type Query @join__type(graph: PRODUCTS)
{
  arguments(
    one: Int,
    two: Boolean @inaccessible,
    notAvailable: InaccessibleInput,
    partially: PartiallyInaccessibleInput,
    partialllyAccessibleEnum: PartiallyAccessibleEnum
  ): Int
  inaccessibleField: Tree @inaccessible

  inaccessibleObject: InaccessibleObject
  inaccessibleScalar: InaccessibleScalar
  inaccessibleEnum: InaccessibleEnum

  partiallyAccessibleUnion: PartiallyAccessibleUnion!
  partiallyAccessibleInterface: PartiallyAccessibleInterface!
  partiallyAccessibleEnum: PartiallyAccessibleEnum!
}

input InaccessibleInput @inaccessible @join__type(graph: PRODUCTS)
{
  id: ID!
}

input PartiallyInaccessibleInput @join__type(graph: PRODUCTS)
{
  yes: Boolean,
  inaccessibleField: Boolean @inaccessible
  inaccessibleScalar: InaccessibleScalar
  inaccessibleEnum: InaccessibleEnum
}

type InaccessibleObject implements PartiallyAccessibleInterface @inaccessible @join__type(graph: PRODUCTS)
{
  id: ID!
}

scalar InaccessibleScalar @inaccessible @join__type(graph: PRODUCTS)

enum InaccessibleEnum @inaccessible @join__type(graph: PRODUCTS)
{
  VALUE
}

enum PartiallyAccessibleEnum @join__type(graph: PRODUCTS)
{
  YES
  NO @inaccessible
}

interface InaccessibleInterface @inaccessible @join__type(graph: PRODUCTS)
{
  id: ID!
}

union InaccessibleUnion @inaccessible @join__type(graph: PRODUCTS)
 = Tree

type Tree implements PartiallyAccessibleInterface & InaccessibleInterface @join__type(graph: PRODUCTS)
{
  id: ID!
}

union PartiallyAccessibleUnion @join__type(graph: PRODUCTS)
 = Tree | InaccessibleObject

interface PartiallyAccessibleInterface @join__type(graph: PRODUCTS)
{
  id: ID!
}
"###;

#[test]
fn inaccessible_should_not_appear_in_introspection() {
    let response = runtime().block_on(async move {
        DeterministicEngine::new(SCHEMA, PATHFINDER_INTROSPECTION_QUERY, &[json!(null)])
            .await
            .execute()
            .await
    });
    assert!(response.errors().is_empty(), "{response}");

    insta::assert_snapshot!(introspection_to_sdl(response.into_data()), @r#"
    enum PartiallyAccessibleEnum {
      YES
    }

    interface PartiallyAccessibleInterface {
      id: ID!
    }

    union PartiallyAccessibleUnion = Tree

    input PartiallyInaccessibleInput {
      yes: Boolean
    }

    type Query {
      arguments(
        one: Int
        partially: PartiallyInaccessibleInput
        partialllyAccessibleEnum: PartiallyAccessibleEnum
      ): Int
      partiallyAccessibleUnion: PartiallyAccessibleUnion!
      partiallyAccessibleInterface: PartiallyAccessibleInterface!
      partiallyAccessibleEnum: PartiallyAccessibleEnum!
    }

    type Tree implements PartiallyAccessibleInterface {
      id: ID!
    }
    "#);
}

#[test]
fn inaccessible_argument() {
    runtime().block_on(async move {
        let response = DeterministicEngine::new(SCHEMA, r#"query { arguments(one: 3, two: true) }"#, &[json!(null)])
            .await
            .execute()
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "The field `Query.arguments` does not have an argument named `two",
              "locations": [
                {
                  "line": 1,
                  "column": 27
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = DeterministicEngine::new(
            SCHEMA,
            r#"query { arguments(one: 3) }"#,
            &[json!({"data": {"arguments": 1}})],
        )
        .await
        .execute()
        .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "arguments": 1
          }
        }
        "#);
    });
}

#[test]
fn inaccessible_object() {
    runtime().block_on(async move {
        let response = DeterministicEngine::new(SCHEMA, r#"query { inaccessibleObject { id } }"#, &[json!(null)])
            .await
            .execute()
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Query does not have a field named 'inaccessibleObject'",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);
    });
}

#[test]
fn inaccessible_scalar() {
    runtime().block_on(async move {
        let response = DeterministicEngine::new(SCHEMA, r#"query { inaccessibleScalar }"#, &[json!(null)])
            .await
            .execute()
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Query does not have a field named 'inaccessibleScalar'",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);
    });
}

#[test]
fn inaccessible_interface() {
    runtime().block_on(async move {
        let response = DeterministicEngine::new(SCHEMA, r#"query { inaccessibleInterface { id } }"#, &[json!(null)])
            .await
            .execute()
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Query does not have a field named 'inaccessibleInterface'",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);
    });
}

#[test]
fn inaccessible_union() {
    runtime().block_on(async move {
        let response =
            DeterministicEngine::new(SCHEMA, r#"query { inaccessibleUnion { __typename } }"#, &[json!(null)])
                .await
                .execute()
                .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Query does not have a field named 'inaccessibleUnion'",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);
    });
}

#[test]
fn inaccessible_field() {
    runtime().block_on(async move {
        let response = DeterministicEngine::new(SCHEMA, r#"query { inaccessibleField { id } }"#, &[json!(null)])
            .await
            .execute()
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Query does not have a field named 'inaccessibleField'",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);
    });
}

#[test]
fn partially_accessible_union() {
    runtime().block_on(async move {
        let response = DeterministicEngine::new(
            SCHEMA,
            r#"query { partiallyAccessibleUnion { ... on InaccessibleObject { id } } }"#,
            &[json!(null)],
        )
        .await
        .execute()
        .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Unknown type named 'InaccessibleObject'",
              "locations": [
                {
                  "line": 1,
                  "column": 40
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = DeterministicEngine::new(
            SCHEMA,
            r#"query { partiallyAccessibleUnion { ... on Tree { id } } }"#,
            &[json!({"data": {"partiallyAccessibleUnion": {"__typename": "Tree", "id": "1"}}})],
        )
        .await
        .execute()
        .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "partiallyAccessibleUnion": {
              "id": "1"
            }
          }
        }
        "#);
    });
}

#[test]
fn partially_accessible_interface() {
    runtime().block_on(async move {
        let response = DeterministicEngine::new(
            SCHEMA,
            r#"query { partiallyAccessibleInterface { ... on InaccessibleObject { id } } }"#,
            &[json!(null)],
        )
        .await
        .execute()
        .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Unknown type named 'InaccessibleObject'",
              "locations": [
                {
                  "line": 1,
                  "column": 44
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = DeterministicEngine::new(
            SCHEMA,
            r#"query { partiallyAccessibleInterface { ... on Tree { id } } }"#,
            &[json!({"data": {"partiallyAccessibleInterface": {"__typename": "Tree", "id": "1"}}})],
        )
        .await
        .execute()
        .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "partiallyAccessibleInterface": {
              "id": "1"
            }
          }
        }
        "#);
    });
}

#[test]
fn inaccessible_input_object() {
    runtime().block_on(async move {
        let response = DeterministicEngine::new(
            SCHEMA,
            r#"query { arguments(notAvailable: { id: 1 }) }"#,
            &[json!(null)],
        )
        .await
        .execute()
        .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "The field `Query.arguments` does not have an argument named `notAvailable",
              "locations": [
                {
                  "line": 1,
                  "column": 19
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);
    });
}

#[test]
fn partially_accessible_input_object() {
    runtime().block_on(async move {
        let response = DeterministicEngine::new(
            SCHEMA,
            r#"query { arguments(partially: { inaccessibleField: true }) }"#,
            &[json!(null)],
        )
        .await
        .execute()
        .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Input object PartiallyInaccessibleInput does not have a field named 'inaccessibleField'",
              "locations": [
                {
                  "line": 1,
                  "column": 30
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = DeterministicEngine::new(
            SCHEMA,
            r#"query { arguments(partially: { inaccessibleScalar: true }) }"#,
            &[json!(null)],
        )
        .await
        .execute()
        .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Input object PartiallyInaccessibleInput does not have a field named 'inaccessibleScalar'",
              "locations": [
                {
                  "line": 1,
                  "column": 30
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = DeterministicEngine::new(
            SCHEMA,
            r#"query { arguments(partially: { inaccessibleEnum: VALUE }) }"#,
            &[json!(null)],
        )
        .await
        .execute()
        .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Input object PartiallyInaccessibleInput does not have a field named 'inaccessibleEnum'",
              "locations": [
                {
                  "line": 1,
                  "column": 30
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = DeterministicEngine::new(
            SCHEMA,
            r#"query { arguments(partially: { yes: true }) }"#,
            &[json!({"data": {"arguments": 1}})],
        )
        .await
        .execute()
        .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "arguments": 1
          }
        }
        "#);
    });
}

#[test]
fn partially_inaccessible_enum_as_input() {
    runtime().block_on(async move {
        let response = DeterministicEngine::new(
            SCHEMA,
            r#"query { arguments(partialllyAccessibleEnum: NO) }"#,
            &[json!(null)],
        )
        .await
        .execute()
        .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Unknown enum value 'NO' for enum PartiallyAccessibleEnum",
              "locations": [
                {
                  "line": 1,
                  "column": 45
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = DeterministicEngine::new(
            SCHEMA,
            r#"query { arguments(partialllyAccessibleEnum: YES) }"#,
            &[json!({"data": {"arguments": 1}})],
        )
        .await
        .execute()
        .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "arguments": 1
          }
        }
        "#);
    });
}

#[test]
fn partially_inaccessible_enum_as_output() {
    runtime().block_on(async move {
        let response = DeterministicEngine::new(
            SCHEMA,
            r#"query { partiallyAccessibleEnum }"#,
            &[json!({"data": {"partiallyAccessibleEnum": "YES"}})],
        )
        .await
        .execute()
        .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "partiallyAccessibleEnum": "YES"
          }
        }
        "#);

        let response = DeterministicEngine::new(
            SCHEMA,
            r#"query { partiallyAccessibleEnum }"#,
            &[json!({"data": {"partiallyAccessibleEnum": "NO"}})],
        )
        .await
        .execute()
        .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": null
        }
        "#);
    });
}

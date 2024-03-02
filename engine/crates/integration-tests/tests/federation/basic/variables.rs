use engine_v2::Engine;
use graphql_mocks::{DisingenuousSchema, EchoSchema, MockGraphQlServer};
use integration_tests::{
    federation::{EngineV2Ext, GraphqlResponse},
    runtime,
};
use serde::Serialize;
use serde_json::json;

#[test]
fn string() {
    roundtrip_test("string", "String!", "hello");
}

#[test]
fn int() {
    roundtrip_test("int", "Int!", 420);
}

#[test]
fn float() {
    roundtrip_test("float", "Float!", 798.0);
}

#[test]
fn id() {
    roundtrip_test(
        "id",
        "ID!",
        "lol-iam-an-id-honestly-what-do-you-mean-i-look-like-a-string",
    );
}

#[test]
fn enum_roundtrip() {
    roundtrip_test("fancyBool", "FancyBool!", "YES");
}

#[test]
fn lists() {
    roundtrip_test(
        "listOfStrings",
        "[String!]!",
        ["hello", "there", "from", "the", "outer", "reaches"],
    );

    roundtrip_test(
        "listOfListOfStrings",
        "[[String!]!]!",
        [["hello", "there", "from"], ["the", "outer", "reaches"]],
    );

    roundtrip_test(
        "optionalListOfOptionalStrings",
        "[String]",
        json!(["hello", "there", "from", null, "the", "outer", "reaches"]),
    );

    roundtrip_test("optionalListOfOptionalStrings", "[String]", json!(null));
}

#[test]
fn input_objects() {
    roundtrip_test(
        "inputObject",
        "InputObj!",
        json!({
            "string": "hello",
            "int": 1,
            "float": 1.0
        }),
    );
    roundtrip_test(
        "inputObject",
        "InputObj!",
        json!({
            "string": "hello",
            "int": 1,
            "float": 1.0
        }),
    );
    roundtrip_test(
        "inputObject",
        "InputObj!",
        json!({
            "string": null,
        }),
    );
    roundtrip_test(
        "inputObject",
        "InputObj!",
        json!({
            "recursiveObject": {"string": null}
        }),
    );
    roundtrip_test(
        "inputObject",
        "InputObj!",
        json!({
            "recursiveObject": {"recursiveObject": {"string": "hello"}}
        }),
    );
    roundtrip_test(
        "inputObject",
        "InputObj!",
        json!({
            "recursiveObject": null
        }),
    );
    roundtrip_test(
        "inputObject",
        "InputObj!",
        json!({
            "recursiveObjectList": null
        }),
    );
    roundtrip_test(
        "inputObject",
        "InputObj!",
        json!({
            "recursiveObjectList": [{"string": "hello"}]
        }),
    );
    roundtrip_test(
        "inputObject",
        "InputObj!",
        json!({
            "recursiveObjectList": [{"recursiveObject": {"string": "hello"}}]
        }),
    );
    roundtrip_test(
        "inputObject",
        "InputObj!",
        json!({
            "recursiveObjectList": [{"recursiveObject": {"fancyBool": "YES"}}]
        }),
    );
}

#[test]
fn test_default_values() {
    let query = r#"query($input: String! = "there") { listOfListOfStrings(input: $input) }"#;
    let input = json!({"input": "hello"});
    assert_eq!(
        run_query(query, &input).into_data()["listOfListOfStrings"],
        json!([["hello"]])
    );

    let input = json!({});
    assert_eq!(
        run_query(query, &input).into_data()["listOfListOfStrings"],
        json!([["there"]])
    );
}

#[test]
fn list_coercion() {
    let query = "query($input: [[String!]!]!) { listOfListOfStrings(input: $input) }";
    let input = json!({"input": "hello"});
    assert_eq!(
        run_query(query, &input).into_data()["listOfListOfStrings"],
        json!([["hello"]])
    );

    let query = "query($input: [String!]!) { listOfStrings(input: $input) }";
    assert_eq!(run_query(query, &input).into_data()["listOfStrings"], json!(["hello"]));

    let query = "query($input: InputObj!) { inputObject(input: $input) }";
    let input = json!({
        "input": {"annoyinglyOptionalStrings": "hello", "recursiveObjectList": {"id": "1"}}
    });
    assert_eq!(
        run_query(query, &input).into_data()["inputObject"],
        json!({
            "annoyinglyOptionalStrings": [["hello"]],
            "recursiveObjectList": [
                {"id": "1"}
            ]
        })
    );

    let input = json!({"input": {"annoyinglyOptionalStrings": null}});
    assert_eq!(
        run_query(query, &input).into_data()["inputObject"],
        json!({
            "annoyinglyOptionalStrings": null
        })
    );
}

#[test]
fn invalid_ints() {
    insta::assert_json_snapshot!(error_test("int", "Int!", 1.5), @r###"
    [
      "Variable $input has an invalid value. Found value 1.5 which cannot be coerced into a Int scalar"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("int", "Int!", "blah"), @r###"
    [
      "Variable $input has an invalid value. Found a String value where we expected a Int scalar"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("int", "Int!", true), @r###"
    [
      "Variable $input has an invalid value. Found a Boolean value where we expected a Int scalar"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("int", "Int!", json!(null)), @r###"
    [
      "Variable $input has an invalid value. Found a null where we expected a Int!"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("int", "Int!", json!({})), @r###"
    [
      "Variable $input has an invalid value. Found a Object value where we expected a Int scalar"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("int", "Int!", json!([])), @r###"
    [
      "Variable $input has an invalid value. Found a List value where we expected a Int scalar"
    ]
    "###);
}

#[test]
fn invalid_strings() {
    insta::assert_json_snapshot!(error_test("string", "String!", 1.5), @r###"
    [
      "Variable $input has an invalid value. Found a Float value where we expected a String scalar"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("string", "String!", 1), @r###"
    [
      "Variable $input has an invalid value. Found a Integer value where we expected a String scalar"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("string", "String!", true), @r###"
    [
      "Variable $input has an invalid value. Found a Boolean value where we expected a String scalar"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("string", "String!", json!(null)), @r###"
    [
      "Variable $input has an invalid value. Found a null where we expected a String!"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("string", "String!", json!({})), @r###"
    [
      "Variable $input has an invalid value. Found a Object value where we expected a String scalar"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("string", "String!", json!([])), @r###"
    [
      "Variable $input has an invalid value. Found a List value where we expected a String scalar"
    ]
    "###);
}

#[test]
fn invalid_floats() {
    insta::assert_json_snapshot!(error_test("float", "Float!", true), @r###"
    [
      "Variable $input has an invalid value. Found a Boolean value where we expected a Float scalar"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("float", "Float!", json!(null)), @r###"
    [
      "Variable $input has an invalid value. Found a null where we expected a Float!"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("float", "Float!", json!({})), @r###"
    [
      "Variable $input has an invalid value. Found a Object value where we expected a Float scalar"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("float", "Float!", json!([])), @r###"
    [
      "Variable $input has an invalid value. Found a List value where we expected a Float scalar"
    ]
    "###);
}

#[test]
fn invalid_id() {
    insta::assert_json_snapshot!(error_test("id", "ID!", true), @r###"
    [
      "Variable $input has an invalid value. Found a Boolean value where we expected a ID scalar"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("id", "ID!", json!(null)), @r###"
    [
      "Variable $input has an invalid value. Found a null where we expected a ID!"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("id", "ID!", json!({})), @r###"
    [
      "Variable $input has an invalid value. Found a Object value where we expected a ID scalar"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("id", "ID!", json!([])), @r###"
    [
      "Variable $input has an invalid value. Found a List value where we expected a ID scalar"
    ]
    "###);
}

#[test]
fn invalid_lists() {
    insta::assert_json_snapshot!(error_test("listOfStrings", "[String!]!", true), @r###"
    [
      "Variable $input has an invalid value. Found a Boolean value where we expected a String scalar"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfStrings", "[String!]!", json!(null)), @r###"
    [
      "Variable $input has an invalid value. Found a null where we expected a [String!]!"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfStrings", "[String!]!", json!([1])), @r###"
    [
      "Variable $input has an invalid value. Found a Integer value where we expected a String scalar at path '.0'"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfStrings", "[String!]!", json!([null])), @r###"
    [
      "Variable $input has an invalid value. Found a null where we expected a String! at path '.0'"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfStrings", "[String!]!", json!(["hello", 1, "there"])), @r###"
    [
      "Variable $input has an invalid value. Found a Integer value where we expected a String scalar at path '.1'"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfStrings", "[String!]!", json!([[null]])), @r###"
    [
      "Variable $input has an invalid value. Found a List value where we expected a String scalar at path '.0'"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfStrings", "[String!]!", json!([["hello"]])), @r###"
    [
      "Variable $input has an invalid value. Found a List value where we expected a String scalar at path '.0'"
    ]
    "###);
}

#[test]
fn invalid_nested_lists() {
    insta::assert_json_snapshot!(error_test("listOfListOfStrings", "[[String!]!]!", true), @r###"
    [
      "Variable $input has an invalid value. Found a Boolean value where we expected a String scalar"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfListOfStrings", "[[String!]!]!", json!(null)), @r###"
    [
      "Variable $input has an invalid value. Found a null where we expected a [[String!]!]!"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfListOfStrings", "[[String!]!]!", json!([1])), @r###"
    [
      "Variable $input has an invalid value. Found a Integer value where we expected a [String!]! at path '.0'"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfListOfStrings", "[[String!]!]!", json!([null])), @r###"
    [
      "Variable $input has an invalid value. Found a null where we expected a [String!]! at path '.0'"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfListOfStrings", "[[String!]!]!", json!([[null]])), @r###"
    [
      "Variable $input has an invalid value. Found a null where we expected a String! at path '.0.0'"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfListOfStrings", "[[String!]!]!", json!([[1]])), @r###"
    [
      "Variable $input has an invalid value. Found a Integer value where we expected a String scalar at path '.0.0'"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfListOfStrings", "[[String!]!]!", json!([["hello", 1, "there"]])), @r###"
    [
      "Variable $input has an invalid value. Found a Integer value where we expected a String scalar at path '.0.1'"
    ]
    "###);
}

#[test]
fn invalid_input_objects() {
    insta::assert_json_snapshot!(
        error_test("inputObject", "InputObj!", json!({"string": 1})),
        @r###"
    [
      "Variable $input has an invalid value. Found a Integer value where we expected a String scalar at path '.string'"
    ]
    "###
    );
    insta::assert_json_snapshot!(
        error_test("inputObject", "InputObj!", json!({"int": "hello"})),
        @r###"
    [
      "Variable $input has an invalid value. Found a String value where we expected a Int scalar at path '.int'"
    ]
    "###
    );
    insta::assert_json_snapshot!(
        error_test("inputObject", "InputObj!", json!({"recursiveObject": {"string": 1}})),
        @r###"
    [
      "Variable $input has an invalid value. Found a Integer value where we expected a String scalar at path '.recursiveObject.string'"
    ]
    "###
    );
    // This one is valid because it gets list coerced
    insta::assert_json_snapshot!(
        error_test("inputObject", "InputObj!", json!({"recursiveObjectList": {}})),
        @"[]"
    );
    insta::assert_json_snapshot!(
        error_test("inputObject", "InputObj!", json!({"recursiveObjectList": [null]})),
        @r###"
    [
      "Variable $input has an invalid value. Found a null where we expected a InputObj! at path '.recursiveObjectList.0'"
    ]
    "###
    );
    // This one is also valid because it gets list coerced
    insta::assert_json_snapshot!(
        error_test("inputObject", "InputObj!", json!({"recursiveObjectList": [{"recursiveObjectList": {}}]})),
        @"[]"
    );
    insta::assert_json_snapshot!(
        error_test("inputObject", "InputObj!", json!({"recursiveObjectList": [{"recursiveObjectList": [null]}]})),
        @r###"
    [
      "Variable $input has an invalid value. Found a null where we expected a InputObj! at path '.recursiveObjectList.0.recursiveObjectList.0'"
    ]
    "###
    );
}

#[test]
fn invalid_enum() {
    insta::assert_json_snapshot!(
        error_test("fancyBool", "FancyBool!", json!("bloo")),
        @r###"
    [
      "Variable $input has an invalid value. Unknown enum value 'bloo' for enum FancyBool"
    ]
    "###
    );
    insta::assert_json_snapshot!(
        error_test("inputObject", "InputObj!", json!({"fancyBool": "blah"})),
        @r###"
    [
      "Variable $input has an invalid value. Unknown enum value 'blah' for enum FancyBool at path '.fancyBool'"
    ]
    "###
    );
}

#[test]
fn multiple_invalid_variables() {
    let query = "query($one: String!, $two: Int!) { string(input: $one) int(input: $two) }";

    let errors = do_error_test(query, json!({"one": true, "two": "hello"}));

    insta::assert_json_snapshot!(errors, @r###"
    [
      "Variable $one has an invalid value. Found a Boolean value where we expected a String scalar",
      "Variable $two has an invalid value. Found a String value where we expected a Int scalar"
    ]
    "###);
}

#[test]
fn variable_uniqueness_validation() {
    insta::assert_json_snapshot!(
        do_error_test(
            "query($one: String!, $one: String!) { one: string(input: $one) }",
            json!({})
        ),
        @r###"
    [
      "There can only be one variable named '$one'"
    ]
    "###
    );
}

#[test]
fn variables_are_input_types_validation() {
    insta::assert_json_snapshot!(
        do_error_test(
            "query($one: Query) { string(input: $one)}",
            json!({})
        ),
        @r###"
    [
      "Variable named '$one' does not have a valid input type. Can only be a scalar, enum or input object. Found: 'Query'."
    ]
    "###
    );
}

#[test]
fn variables_are_defined_validation() {
    insta::assert_json_snapshot!(
        do_error_test(
            "query { string(input: $one) }",
            json!({})
        ),
        @r###"
    [
      "Unknown variable $one"
    ]
    "###
    );

    insta::assert_json_snapshot!(
        do_error_test(
            "query MyOperation { string(input: $one) }",
            json!({})
        ),
        @r###"
    [
      "Unknown variable $one"
    ]
    "###
    );

    insta::assert_json_snapshot!(
        do_error_test(
            r#"
                query Blah {
                    ...MyFragment
                }

                fragment MyFragment on Query {
                    string(input: $one)
                }
            "#,
            json!({})
        ),
        @r###"
    [
      "Unknown variable $one"
    ]
    "###
    );

    insta::assert_json_snapshot!(
        do_error_test(
            r#"
            query {
                ... {
                    string(input: $one)
                }
            }
            "#,
            json!({})
        ),
        @r###"
    [
      "Unknown variable $one"
    ]
    "###
    );
}

#[test]
fn variables_are_used_validation() {
    insta::assert_json_snapshot!(
        do_error_test(
            r#"query($one: String!) { string(input: "hello") }"#,
            json!({})
        ),
        @r###"
    [
      "Variable '$one' is not used"
    ]
    "###
    );

    insta::assert_json_snapshot!(
        do_error_test(
            r#"query Blah($one: String!) { string(input: "hello") }"#,
            json!({})
        ),
        @r###"
    [
      "Variable '$one' is not used by operation 'Blah'"
    ]
    "###
    );

    // This one should pass because it's used transient
    insta::assert_json_snapshot!(
        do_error_test(
            r#"
                query($one: String!) {
                    ...MyFragment
                }

                fragment MyFragment on Query {
                    string(input: $one)
                }
            "#,
            json!({"one": "hello"})
        ),
        @"[]"
    );
}

#[test]
fn undefined_variable() {
    let query = "query($var: String) { echo(input: { string: $var, int: 10 }) }";
    let response = runtime().block_on({
        async move {
            let echo_mock = MockGraphQlServer::new(DisingenuousSchema::with_sdl(
                r#"
                scalar JSON

                type Query {
                    echo(input: InputObj!): JSON
                }

                input InputObj {
                    string: String
                    int: Int!
                }
                "#,
            ))
            .await;

            let engine = Engine::builder().with_schema("schema", &echo_mock).await.finish().await;

            engine.execute(query).variables(json!({})).await
        }
    });

    // FIXME: This is incorrect with respect to the spec. "string" field should be undefined and
    // not present. But async-graphql doesn't respect it... So until we support something else than
    // GraphQL (UDFs typically), we're stuck.
    // https://spec.graphql.org/October2021/#sec-Input-Objects.Input-Coercion
    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "echo": {
          "string": null,
          "int": 10
        }
      }
    }
    "###);
}

#[track_caller]
fn roundtrip_test<T>(field: &str, ty: &str, input: T)
where
    T: Serialize,
{
    let query = format!("query($input: {ty}) {{ {field}(input: $input) }}");

    do_roundtrip_test(field, &query, serde_json::to_value(input).unwrap());
}

#[track_caller]
fn do_roundtrip_test(field: &str, query: &str, input: serde_json::Value) {
    let response = run_query(query, &json!({"input": input}));

    assert_eq!(response.into_data()[field], input);
}

fn error_test<T>(field: &str, ty: &str, input: T) -> Vec<String>
where
    T: Serialize,
{
    let query = format!("query($input: {ty}) {{ {field}(input: $input) }}");

    do_error_test(&query, json!({"input": input}))
}

fn do_error_test(query: &str, input: serde_json::Value) -> Vec<String> {
    run_query(query, &input)
        .errors()
        .iter()
        .map(|error| error["message"].as_str().expect("message to be a string").to_string())
        .collect()
}

fn run_query(query: &str, input: &serde_json::Value) -> GraphqlResponse {
    runtime().block_on({
        async move {
            let echo_mock = MockGraphQlServer::new(EchoSchema::default()).await;

            let engine = Engine::builder().with_schema("schema", &echo_mock).await.finish().await;

            engine.execute(query).variables(input).await
        }
    })
}

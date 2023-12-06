use engine_v2::Engine;
use integration_tests::{
    federation::{EngineV2Ext, GraphqlResponse},
    mocks::graphql::EchoSchema,
    runtime, MockGraphQlServer,
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
    let query = r#"query($input: String = "there") { listOfListOfStrings(input: $input) }"#;
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
      "Variable $input got an invalid value: found a Float value where we expected a Int scalar at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("int", "Int!", "blah"), @r###"
    [
      "Variable $input got an invalid value: found a String value where we expected a Int scalar at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("int", "Int!", true), @r###"
    [
      "Variable $input got an invalid value: found a Boolean value where we expected a Int scalar at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("int", "Int!", json!(null)), @r###"
    [
      "Variable $input got an invalid value: found a null where we expected a Int! at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("int", "Int!", json!({})), @r###"
    [
      "Variable $input got an invalid value: found a Object value where we expected a Int scalar at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("int", "Int!", json!([])), @r###"
    [
      "Variable $input got an invalid value: found a List value where we expected a Int scalar at $input"
    ]
    "###);
}

#[test]
fn invalid_strings() {
    insta::assert_json_snapshot!(error_test("string", "String!", 1.5), @r###"
    [
      "Variable $input got an invalid value: found a Float value where we expected a String scalar at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("string", "String!", 1), @r###"
    [
      "Variable $input got an invalid value: found a Integer value where we expected a String scalar at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("string", "String!", true), @r###"
    [
      "Variable $input got an invalid value: found a Boolean value where we expected a String scalar at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("string", "String!", json!(null)), @r###"
    [
      "Variable $input got an invalid value: found a null where we expected a String! at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("string", "String!", json!({})), @r###"
    [
      "Variable $input got an invalid value: found a Object value where we expected a String scalar at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("string", "String!", json!([])), @r###"
    [
      "Variable $input got an invalid value: found a List value where we expected a String scalar at $input"
    ]
    "###);
}

#[test]
fn invalid_floats() {
    insta::assert_json_snapshot!(error_test("float", "Float!", true), @r###"
    [
      "Variable $input got an invalid value: found a Boolean value where we expected a Float scalar at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("float", "Float!", json!(null)), @r###"
    [
      "Variable $input got an invalid value: found a null where we expected a Float! at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("float", "Float!", json!({})), @r###"
    [
      "Variable $input got an invalid value: found a Object value where we expected a Float scalar at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("float", "Float!", json!([])), @r###"
    [
      "Variable $input got an invalid value: found a List value where we expected a Float scalar at $input"
    ]
    "###);
}

#[test]
fn invalid_id() {
    insta::assert_json_snapshot!(error_test("id", "ID!", true), @r###"
    [
      "Variable $input got an invalid value: found a Boolean value where we expected a ID scalar at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("id", "ID!", json!(null)), @r###"
    [
      "Variable $input got an invalid value: found a null where we expected a ID! at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("id", "ID!", json!({})), @r###"
    [
      "Variable $input got an invalid value: found a Object value where we expected a ID scalar at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("id", "ID!", json!([])), @r###"
    [
      "Variable $input got an invalid value: found a List value where we expected a ID scalar at $input"
    ]
    "###);
}

#[test]
fn invalid_lists() {
    insta::assert_json_snapshot!(error_test("listOfStrings", "[String!]!", true), @r###"
    [
      "Variable $input got an invalid value: found a Boolean value where we expected a String scalar at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfStrings", "[String!]!", json!(null)), @r###"
    [
      "Variable $input got an invalid value: found a null where we expected a [String!]! at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfStrings", "[String!]!", json!([1])), @r###"
    [
      "Variable $input got an invalid value: found a Integer value where we expected a String scalar at $input.0"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfStrings", "[String!]!", json!([null])), @r###"
    [
      "Variable $input got an invalid value: found a null where we expected a String! at $input.0"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfStrings", "[String!]!", json!(["hello", 1, "there"])), @r###"
    [
      "Variable $input got an invalid value: found a Integer value where we expected a String scalar at $input.1"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfStrings", "[String!]!", json!([[null]])), @r###"
    [
      "Variable $input got an invalid value: found a List value where we expected a String scalar at $input.0"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfStrings", "[String!]!", json!([["hello"]])), @r###"
    [
      "Variable $input got an invalid value: found a List value where we expected a String scalar at $input.0"
    ]
    "###);
}

#[test]
fn invalid_nested_lists() {
    insta::assert_json_snapshot!(error_test("listOfListOfStrings", "[[String!]!]!", true), @r###"
    [
      "Variable $input got an invalid value: found a Boolean value where we expected a String scalar at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfListOfStrings", "[[String!]!]!", json!(null)), @r###"
    [
      "Variable $input got an invalid value: found a null where we expected a [[String!]!]! at $input"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfListOfStrings", "[[String!]!]!", json!([1])), @r###"
    [
      "Variable $input got an invalid value: found a Integer value where we expected a [String!]! at $input.0"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfListOfStrings", "[[String!]!]!", json!([null])), @r###"
    [
      "Variable $input got an invalid value: found a null where we expected a [String!]! at $input.0"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfListOfStrings", "[[String!]!]!", json!([[null]])), @r###"
    [
      "Variable $input got an invalid value: found a null where we expected a String! at $input.0.0"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfListOfStrings", "[[String!]!]!", json!([[1]])), @r###"
    [
      "Variable $input got an invalid value: found a Integer value where we expected a String scalar at $input.0.0"
    ]
    "###);
    insta::assert_json_snapshot!(error_test("listOfListOfStrings", "[[String!]!]!", json!([["hello", 1, "there"]])), @r###"
    [
      "Variable $input got an invalid value: found a Integer value where we expected a String scalar at $input.0.1"
    ]
    "###);
}

#[test]
fn invalid_input_objects() {
    insta::assert_json_snapshot!(
        error_test("inputObject", "InputObj!", json!({"string": 1})),
        @r###"
    [
      "Variable $input got an invalid value: found a Integer value where we expected a String scalar at $input.string"
    ]
    "###
    );
    insta::assert_json_snapshot!(
        error_test("inputObject", "InputObj!", json!({"int": "hello"})),
        @r###"
    [
      "Variable $input got an invalid value: found a String value where we expected a Int scalar at $input.int"
    ]
    "###
    );
    insta::assert_json_snapshot!(
        error_test("inputObject", "InputObj!", json!({"recursiveObject": {"string": 1}})),
        @r###"
    [
      "Variable $input got an invalid value: found a Integer value where we expected a String scalar at $input.recursiveObject.string"
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
      "Variable $input got an invalid value: found a null where we expected a InputObj! at $input.recursiveObjectList.0"
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
      "Variable $input got an invalid value: found a null where we expected a InputObj! at $input.recursiveObjectList.0.recursiveObjectList.0"
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
      "Variable $input got an invalid value: found the value 'bloo' value where we expected a value of the 'FancyBool' enum at $input"
    ]
    "###
    );
    insta::assert_json_snapshot!(
        error_test("inputObject", "InputObj!", json!({"fancyBool": "blah"})),
        @r###"
    [
      "Variable $input got an invalid value: found the value 'blah' value where we expected a value of the 'FancyBool' enum at $input.fancyBool"
    ]
    "###
    );
}

#[test]
fn multiple_invalid_variables() {
    let query = "query($one: String!, $two: Int!) { string(input: $one) int(input: $two) }";

    let errors = run_query(query, &json!({"one": true, "two": "hello"}))
        .errors()
        .iter()
        .map(|error| error["message"].as_str().expect("message to be a string").to_string())
        .collect::<Vec<_>>();

    insta::assert_json_snapshot!(errors, @r###"
    [
      "Variable $one got an invalid value: found a Boolean value where we expected a String scalar at $one",
      "Variable $two got an invalid value: found a String value where we expected a Int scalar at $two"
    ]
    "###);
}

fn roundtrip_test<T>(field: &str, ty: &str, input: T)
where
    T: Serialize,
{
    let query = format!("query($input: {ty}) {{ {field}(input: $input) }}");

    do_roundtrip_test(field, &query, serde_json::to_value(input).unwrap());
}

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

            let engine = Engine::build().with_schema("schema", &echo_mock).await.finish();

            engine.execute(query).variables(input).await
        }
    })
}

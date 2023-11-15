use engine_v2::Engine;
use integration_tests::{federation::EngineV2Ext, mocks::graphql::EchoSchema, runtime, MockGraphQlServer};
use serde::Serialize;
use serde_json::json;

#[test]
#[ignore]
fn string() {
    roundtrip_test("string", "String!", "hello");
}

#[test]
#[ignore]
fn int() {
    roundtrip_test("int", "Int!", 420);
}

#[test]
#[ignore]
fn float() {
    roundtrip_test("int", "Int!", 0.1);
}

#[test]
#[ignore]
fn id() {
    roundtrip_test(
        "id",
        "ID!",
        "lol-iam-an-id-honestly-what-do-you-mean-i-look-like-a-string",
    );
}

#[test]
#[ignore]
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
        json!([["hello", "there", "from"], ["the", "outer", "reaches"]]),
    );
}

#[test]
#[ignore]
fn input_objects() {
    todo!()
}

#[test]
#[ignore]
fn list_coercion() {
    let query = "query(input: String!) { listOfListOfStrings(input: $input) }";
    let input = json!("hello");

    let response = runtime()
        .block_on({
            let input = input.clone();
            async move {
                let echo_mock = MockGraphQlServer::new(EchoSchema::default()).await;

                let engine = Engine::build().with_schema("schema", echo_mock).await.finish();

                engine.execute(query).variables(json!({"input": input})).await
            }
        })
        .unwrap();

    assert_eq!(response["data"], json!([["hello"]]));
}

#[test]
#[ignore]
fn errors_on_type_mismatches() {
    // This is kinda hard to implement without knowing how errors are returned.
    // So just leaving it here as a TODO
    todo!()
}

fn roundtrip_test<T>(field: &str, ty: &str, input: T)
where
    T: Serialize,
{
    let query = format!("query($input: {ty}) {{ {field}(input: $input) }}");

    do_roundtrip_test(&query, serde_json::to_value(input).unwrap());
}

fn do_roundtrip_test(query: &str, input: serde_json::Value) {
    let response = runtime()
        .block_on({
            let input = input.clone();
            async move {
                let echo_mock = MockGraphQlServer::new(EchoSchema::default()).await;

                let engine = Engine::build().with_schema("schema", echo_mock).await.finish();

                engine.execute(query).variables(json!({"input": input})).await
            }
        })
        .unwrap();

    // I'm not certain this is the right assert since execute doesn't actually return
    // anything right now.
    // But we can fix that later.
    assert_eq!(response["data"], input);
}

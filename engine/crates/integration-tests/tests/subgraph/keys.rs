use integration_tests::{runtime, udfs::RustUdfs, EngineBuilder, ResponseExt};
use runtime::udf::{CustomResolverRequestPayload, UdfResponse};
use serde_json::{json, Value};

#[test]
fn test_multi_field_keys() {
    let schema = r#"
        extend schema @federation(version: "2.3")

        extend type Query {
            todo(list: ID!, id: ID!): Todo @resolver(name: "todo")
        }

        type Todo @key(fields: "list id", select: "todo(list: $list, id: $id)") {
            list: ID!
            id: ID!
            name: String!
        }
    "#;

    runtime().block_on(async {
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver("todo", |input: CustomResolverRequestPayload| {
                let id = input.arguments["id"].as_str().unwrap();
                let list = input.arguments["list"].as_str().unwrap();
                Ok(UdfResponse::Success(json!({
                    "id": id,
                    "list": list,
                    "name": format!("Todo {id} in list {list}")
                })))
            }))
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r"
                    query($repr: _Any!) {
                        _entities(representations: [$repr]) {
                            __typename
                            ... on Todo {
                                name
                            }
                        }
                    }
                ",
                )
                .variables(json!({"repr": {
                    "__typename": "Todo",
                    "id": "123",
                    "list": "456"
                }}))
                .await
                .into_data::<Value>(),
                @r###"
        {
          "_entities": [
            {
              "__typename": "Todo",
              "name": "Todo 123 in list 456"
            }
          ]
        }
        "###
        );
    });
}

#[test]
fn test_composite_keys() {
    let schema = r#"
        extend schema @federation(version: "2.3")

        extend type Query {
            todo(input: GetTodoInput!): Todo @resolver(name: "todo")
        }

        input GetTodoInput {
            id: ID!
            list: GetTodoInputList!
        }

        input GetTodoInputList {
            id: ID!
        }

        type Todo @key(fields: "id list { id }", select: "todo(input: {id: $id, list: $list})") {
            list: ID!
            id: ID!
            name: String!
        }
    "#;

    runtime().block_on(async {
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver("todo", |input: CustomResolverRequestPayload| {
                let input = input.arguments["input"].as_object().unwrap();
                let id = input["id"].as_str().unwrap();
                let list = input["list"]["id"].as_str().unwrap();
                Ok(UdfResponse::Success(json!({
                    "id": id,
                    "list": list,
                    "name": format!("Todo {id} in list {list}")
                })))
            }))
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r"
                    query($repr: _Any!) {
                        _entities(representations: [$repr]) {
                            __typename
                            ... on Todo {
                                name
                            }
                        }
                    }
                ",
                )
                .variables(json!({"repr": {
                    "__typename": "Todo",
                    "id": "123",
                    "list": {
                        "id": "456"
                    }
                }}))
                .await
                .into_data::<Value>(),
                @r###"
        {
          "_entities": [
            {
              "__typename": "Todo",
              "name": "Todo 123 in list 456"
            }
          ]
        }
        "###
        );
    });
}

#[test]
fn test_repeated_keys() {
    let schema = r#"
        extend schema @federation(version: "2.3")

        extend type Query {
            todoByListAndId(list: ID!, id: ID!): Todo @resolver(name: "todoByListAndId")
            todoByUnique(uid: ID!): Todo @resolver(name: "todoByUnique")
        }

        type Todo
            @key(fields: "list id", select: "todoByListAndId(list: $list, id: $id)")
            @key(fields: "uid", select: "todoByUnique(uid: $uid)")
        {
            list: ID!
            id: ID!
            uid: ID!
            name: String!
        }
    "#;

    runtime().block_on(async {
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new()
                    .resolver("todoByListAndId", |input: CustomResolverRequestPayload| {
                        let id = input.arguments["id"].as_str().unwrap();
                        let list = input.arguments["list"].as_str().unwrap();
                        Ok(UdfResponse::Success(json!({
                            "id": id,
                            "list": list,
                            "uid": id,
                            "name": format!("Todo {id} in list {list}")
                        })))
                    })
                    .resolver("todoByUnique", |input: CustomResolverRequestPayload| {
                        let uid = input.arguments["uid"].as_str().unwrap();
                        Ok(UdfResponse::Success(json!({
                            "id": uid,
                            "list": uid,
                            "uid": uid,
                            "name": format!("Todo with unique id {uid}")
                        })))
                    }),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r"
                    query($reprs: [_Any!]!) {
                        _entities(representations: $reprs) {
                            __typename
                            ... on Todo {
                                name
                            }
                        }
                    }
                ",
                )
                .variables(json!({"reprs": [
                    { "__typename": "Todo", "id": "123", "list": "456" },
                    { "__typename": "Todo", "uid": "789" },
                ]}))
                .await
                .into_data::<Value>(),
                @r###"
        {
          "_entities": [
            {
              "__typename": "Todo",
              "name": "Todo 123 in list 456"
            },
            {
              "__typename": "Todo",
              "name": "Todo with unique id 789"
            }
          ]
        }
        "###
        );
    });
}

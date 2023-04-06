#![allow(clippy::too_many_lines)]
mod utils;

use serde_json::Value;
use utils::environment::Environment;

#[rstest::rstest]
#[case(
    r#"
        type Post @model {
            text: String! @resolver(name: "return-text")
        }
    "#,
    "return-text.js",
    r#"
        export default function Resolver({ parent, args, context, info }) {
            return "Lorem ipsum dolor sit amet";
        }
    "#,
    &[
        ("query GetPost($id: ID!) { post(by: { id: $id }) { text } }", "data.post.text")
    ],
)]
#[case(
    r#"
        type Post @model {
            fetchResult: JSON! @resolver(name: "fetch-grafbase-graphql")
        }
    "#,
    "fetch-grafbase-graphql.js",
    r#"
        export default function Resolver({ parent, args, context, info }) {
            return fetch('https://api.grafbase.com/graphql', {
                headers: {
                    'content-type': 'application/json'
                },
                method: 'POST',
                body: JSON.stringify({ query: '{ __typename }' })
            });
        }
    "#,
    &[
        ("query GetPost($id: ID!) { post(by: { id: $id }) { fetchResult } }", "data.post.fetchResult")
    ],
)]
#[case(
    r#"
        type Post @model {
            variable(name: String!): String @resolver(name: "return-env-variable")
        }
    "#,
    "return-env-variable.js",
    r#"
        export default function Resolver({ parent, args, context, info }) {
            return process.env[args.name] || null;
        }
    "#,
    &[
        (
            r#"
                query GetPost($id: ID!) {
                    post(by: { id: $id }) {
                        variable(name: "GRAFBASE_ENV")
                    }
                }
            "#,
            "data.post.variable"
        ),
        (
            r#"
                query GetPost($id: ID!) {
                    post(by: { id: $id }) {
                        variable(name: "MY_OWN_VARIABLE")
                    }
                }
            "#,
            "data.post.variable"
        ),
    ],
)]
#[case(
    r#"
        type Post @model {
            variable: String! @resolver(name: "return-env-variable")
        }
    "#,
    "return-env-variable.js",
    r#"
        const value = process.env["MY_OWN_VARIABLE"];

        export default function Resolver({ parent, args, context, info }) {
            return value;
        }
    "#,
    &[
        (
            r#"
                query GetPost($id: ID!) {
                    post(by: { id: $id }) {
                        variable
                    }
                }
            "#,
            "data.post.variable"
        ),
    ],
)]
#[case(
    r#"
        type Post @model {
            object: JSON! @resolver(name: "nested/return-object")
        }
    "#,
    "nested/return-object.ts",
    r#"
        export default function Resolver({ parent, args, context, info }) {
            const returnValue: any = { a: 123, b: "Hello" };
            return returnValue;
        }
    "#,
    &[
        ("query GetPost($id: ID!) { post(by: { id: $id }) { object } }", "data.post.object")
    ],
)]
#[cfg_attr(target_os = "windows", ignore)]
fn test_field_resolver(
    #[case] schema: &str,
    #[case] resolver_name: &str,
    #[case] resolver_contents: &str,
    #[case] queries: &[(&str, &str)],
) {
    let mut env = Environment::init();
    env.grafbase_init();
    std::fs::write(env.directory.join("grafbase/.env"), "MY_OWN_VARIABLE=test_value").unwrap();
    env.write_schema(schema);
    env.write_resolver(resolver_name, resolver_contents);
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(60, 300);

    // Create.
    let response = client
        .gql::<Value>(
            r#"
                mutation {
                    postCreate(
                        input: {}
                    ) {
                        post {
                            id
                        }
                    }
                }
            "#,
        )
        .send();
    let post_id = dot_get!(response, "data.postCreate.post.id", String);

    // Run queries.
    for (index, (query_contents, path)) in queries.iter().enumerate() {
        let response = client
            .gql::<Value>(query_contents.to_owned())
            .variables(serde_json::json!({ "id": post_id }))
            .send();
        let value = dot_get_opt!(response, path, serde_json::Value).unwrap_or_default();
        if let Some(value) = value.as_str() {
            insta::assert_snapshot!(format!("{resolver_name}_{index}"), value);
        } else {
            insta::assert_json_snapshot!(format!("{resolver_name}_{index}"), value);
        }
    }
}

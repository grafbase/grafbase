#![allow(clippy::too_many_lines)]
mod utils;

use serde_json::Value;
use utils::environment::Environment;

#[rstest::rstest]
#[case(
    include_str!("./graphql/custom-resolvers/schema-with-text.graphql"),
    "return-text.js",
    include_str!("./resolvers/return-text.js"),
    &[
        (include_str!("./graphql/custom-resolvers/query-with-text.graphql"), "data.post.text")
    ],
)]
#[case(
    include_str!("./graphql/custom-resolvers/schema-with-fetch-result.graphql"),
    "fetch-grafbase-graphql.js",
    include_str!("./resolvers/fetch-grafbase-graphql.js"),
    &[
        (include_str!("./graphql/custom-resolvers/query-with-fetch-result.graphql"), "data.post.fetchResult")
    ],
)]
#[case(
    include_str!("./graphql/custom-resolvers/schema-with-env-variable.graphql"),
    "return-env-variable.js",
    include_str!("./resolvers/return-env-variable.js"),
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
    include_str!("./graphql/custom-resolvers/schema-with-json-in-ts.graphql"),
    "nested/return-object.ts",
    include_str!("./resolvers/return-object-in-ts.ts"),
    &[
        (include_str!("./graphql/custom-resolvers/query-with-object.graphql"), "data.post.object")
    ],
)]
#[cfg_attr(target_os = "windows", ignore)]
fn test_resolver(
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
    let client = env.create_client();
    client.poll_endpoint(60, 300);

    // Create.
    let response = client
        .gql::<Value>(include_str!("./graphql/custom-resolvers/create.graphql"))
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

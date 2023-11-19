#![allow(unused_crate_dependencies)]
#![allow(clippy::too_many_lines)]
mod utils;

use backend::project::ConfigType;
use rstest_reuse::{self, apply, template};
use serde_json::Value;
use utils::environment::Environment;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Deserialize, strum::Display)]
#[strum(serialize_all = "lowercase")]
enum JavaScriptPackageManager {
    Npm,
    Pnpm,
    Yarn,
}

#[rstest::rstest]
#[case(
    1,
    r#"
        type Post @model {
            title: String!
            text: String! @resolver(name: "return-text")
        }
    "#,
    "return-text.js",
    r#"
        export default function Resolver(parent, args, context, info) {
            return "Lorem ipsum dolor sit amet";
        }
    "#,
    &[
        ("query GetPost($id: ID!) { post(by: { id: $id }) { text } }", "data.post.text")
    ],
    None,
)]
#[case(
    2,
    r#"
        type Post @model {
            title: String!
            fetchResult: JSON! @resolver(name: "fetch-grafbase-graphql")
        }
    "#,
    "fetch-grafbase-graphql.js",
    r"
        export default function Resolver(parent, args, context, info) {
            return fetch('https://api.grafbase.com/graphql', {
                headers: {
                    'content-type': 'application/json'
                },
                method: 'POST',
                body: JSON.stringify({ query: '{ __typename }' })
            });
        }
    ",
    &[
        ("query GetPost($id: ID!) { post(by: { id: $id }) { fetchResult } }", "data.post.fetchResult")
    ],
    None,
)]
#[case(
    3,
    r#"
        type Post @model {
            title: String!
            variable(name: String!): String @resolver(name: "return-env-variable")
        }
    "#,
    "return-env-variable.js",
    r"
        export default function Resolver(parent, args, context, info) {
            return process.env[args.name] || null;
        }
    ",
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
    None,
)]
#[case(
    4,
    r#"
        type Post @model {
            title: String!
            variable: String! @resolver(name: "return-env-variable")
        }
    "#,
    "return-env-variable.js",
    r#"
        const value = process.env["MY_OWN_VARIABLE"];

        export default function Resolver(parent, args, context, info) {
            return value;
        }
    "#,
    &[
        (
            r"
                query GetPost($id: ID!) {
                    post(by: { id: $id }) {
                        variable
                    }
                }
            ",
            "data.post.variable"
        ),
    ],
    None,
)]
#[case(
    5,
    r#"
        type Post @model {
            title: String!
            object: JSON! @resolver(name: "nested/return-object")
        }
    "#,
    "nested/return-object.ts",
    r#"
        export default function Resolver(parent, args, context, info) {
            const returnValue: any = { a: 123, b: "Hello" };
            return returnValue;
        }
    "#,
    &[
        ("query GetPost($id: ID!) { post(by: { id: $id }) { object } }", "data.post.object")
    ],
    None,
)]
#[case(
    6,
    r#"
        type Post @model {
            title: String!
            title2: String! @resolver(name: "return-title")
        }
    "#,
    "return-title.js",
    r"
        export default function Resolver(parent, args, context, info) {
            return parent.title;
        }
    ",
    &[
        ("query GetPost($id: ID!) { post(by: { id: $id }) { title2 } }", "data.post.title2")
    ],
    None,
)]
#[case(
    7,
    r#"
        type Post @model {
            title: String!
            headerValue(name: String!): String @resolver(name: "return-header-value")
        }
    "#,
    "return-header-value.js",
    r"
        export default function Resolver(parent, args, context, info) {
            return context.request.headers[args.name];
        }
    ",
    &[
        ("query GetPost($id: ID!) { post(by: { id: $id }) { headerValue(name: \"x-test-header\") } }", "data.post.headerValue")
    ],
    None,
)]
#[case(
    8,
    r#"
        type Post @model {
            title: String!
            isTitlePalindrome: Boolean! @resolver(name: "resolver")
        }
    "#,
    "resolver.js",
    r"
        const isPalindrome = require('is-palindrome');
        export default function Resolver(parent, args, context, info) {
            return isPalindrome(parent.title);
        }
    ",
    &[
        ("query GetPost($id: ID!) { post(by: { id: $id }) { isTitlePalindrome } }", "data.post.isTitlePalindrome")
    ],
    Some((JavaScriptPackageManager::Npm, r#"
        {
            "name": "my-package",
            "dependencies": {
                "is-palindrome": "^0.3.0"
            }
        }
    "#))
)]
#[case(
    9,
    r#"
        type Post @model {
            title: String!
            isTitlePalindrome: Boolean! @resolver(name: "resolver")
        }
    "#,
    "resolver.js",
    r"
        const isPalindrome = require('is-palindrome');
        export default function Resolver(parent, args, context, info) {
            return isPalindrome(parent.title);
        }
    ",
    &[
        ("query GetPost($id: ID!) { post(by: { id: $id }) { isTitlePalindrome } }", "data.post.isTitlePalindrome")
    ],
    Some((JavaScriptPackageManager::Pnpm, r#"
        {
            "dependencies": {
                "is-palindrome": "^0.3.0"
            },
            "packageManager": "^pnpm@8.2.0"
        }
    "#))
)]
#[case(
    10,
    r#"
        type Post @model {
            title: String!
            isTitlePalindrome: Boolean! @resolver(name: "resolver")
        }
    "#,
    "resolver.js",
    r"
        const isPalindrome = require('is-palindrome');
        export default function Resolver(parent, args, context, info) {
            return isPalindrome(parent.title);
        }
    ",
    &[
        ("query GetPost($id: ID!) { post(by: { id: $id }) { isTitlePalindrome } }", "data.post.isTitlePalindrome")
    ],
    Some((JavaScriptPackageManager::Yarn, r#"
        {
            "dependencies": {
                "is-palindrome": "^0.3.0"
            },
            "packageManager": "^yarn@1.22.0"
        }
    "#))
)]
fn test_field_resolver(
    #[case] case_index: usize,
    #[case] schema: &str,
    #[case] resolver_name: &str,
    #[case] resolver_contents: &str,
    #[case] queries: &[(&str, &str)],
    #[case] package_json: Option<(JavaScriptPackageManager, &str)>,
    #[values(("./", "./"), ("./grafbase", "../"))] variant: (&str, &str),
) {
    let (subdirectory_path, package_json_path) = variant;
    let mut env = Environment::init_in_subdirectory(subdirectory_path);
    env.grafbase_init(ConfigType::GraphQL);
    std::fs::write(
        env.directory_path.join(subdirectory_path).join(".env"),
        "MY_OWN_VARIABLE=test_value",
    )
    .unwrap();
    env.write_schema(schema);
    env.write_resolver(resolver_name, resolver_contents);
    if let Some((package_manager, package_json)) = package_json {
        env.write_file(
            std::path::Path::new(package_json_path).join("package.json"),
            package_json,
        );
        // Use `which` to work-around weird path search issues on Windows.
        // See https://github.com/rust-lang/rust/issues/37519.
        let program_path = which::which(package_manager.to_string()).expect("command must be found");
        let command = duct::cmd!(program_path, "install");
        command.dir(&env.directory_path).run().expect("should have succeeded");
    }
    env.grafbase_dev();
    let client = env
        .create_client_with_options(utils::client::ClientOptionsBuilder::default().http_timeout(60).build())
        .with_api_key();
    client.poll_endpoint(120, 250);

    // Create.
    let response = client
        .gql::<Value>(
            r#"
                mutation {
                    postCreate(
                        input: {
                            title: "Hello"
                        }
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
            .header("x-test-header", "test-value")
            .variables(serde_json::json!({ "id": post_id }))
            .send();
        let errors = dot_get_opt!(response, "errors", Vec::<serde_json::Value>).unwrap_or_default();
        assert!(errors.is_empty(), "Error response: {errors:?}");
        let value = dot_get_opt!(response, path, serde_json::Value).unwrap_or_default();
        let snapshot_name = format!("field_resolver_{case_index}_{index}", index = index + 1);
        if let Some(value) = value.as_str() {
            insta::assert_snapshot!(snapshot_name, value);
        } else {
            insta::assert_json_snapshot!(snapshot_name, value);
        }
    }
}

#[template]
#[rstest::rstest]
#[case(
    1,
    r#"
        extend type Query {
            hello: String @resolver(name: "hello")
        }
    "#,
    &[
        (
            "hello.js",
            r"
                export default function Resolver(parent, args, context, info) {
                    return 'Hello World!';
                }
            "
        )
    ],
    &[
        (
            r"
                {
                    hello
                }
            ",
            "data.hello"
        ),
    ],
)]
#[case(
    2,
    r#"
        extend type Mutation {
            stringToNumber(string: String!): Int @resolver(name: "string-to-number")
        }
    "#,
    &[
        (
            "string-to-number.js",
            r"
                export default function Resolver(parent, args, context, info) {
                    return +args.string;
                }
            ",
        )
    ],
    &[
        (
            r#"
                mutation {
                    stringToNumber(string: "123")
                }
            "#,
            "data.stringToNumber"
        ),
    ],
)]
#[case(
    3,
    r#"
        extend type Query {
            hello: String @resolver(name: "hello")
        }
    "#,
    &[
        (
            "another-file.js",
            r"
                export function helper(parent, args, context, info) {
                    return 'Hello World!';
                }
            "
        ),
        (
            "hello.js",
            r"
                import { helper } from './another-file';

                export default function Resolver(parent, args, context, info) {
                    return helper(parent, args, context, info);
                }
            ",
        )
    ],
    &[
        (
            r"
                {
                    hello
                }
            ",
            "data.hello"
        ),
    ],
)]
#[case(
    4,
    r#"
        extend type Query {
            hello: String @resolver(name: "hello")
        }
    "#,
    &[
        (
            "hello.js",
            r#"
                export default function Resolver(parent, args, context, info) {
                    console.log("Hello")
                    return "Hello"
                }
            "#,
        )
    ],
    &[
        (
            r"
                {
                    hello
                }
            ",
            "data.hello"
        ),
    ],
)]
fn test_query_mutation_resolver(
    #[case] case_index: usize,
    #[case] schema: &str,
    #[case] resolver_files: &[(&str, &str)],
    #[case] queries: &[(&str, &str)],
) {
}

#[apply(test_query_mutation_resolver)]
fn test_query_mutation_resolver_dev(
    case_index: usize,
    schema: &str,
    resolver_files: &[(&str, &str)],
    queries: &[(&str, &str)],
) {
    let mut env = Environment::init();
    env.grafbase_init(ConfigType::GraphQL);
    env.write_schema(schema);
    for (file_name, file_contents) in resolver_files {
        env.write_resolver(file_name, file_contents);
    }

    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(60, 300);

    // Run queries.
    for (index, (query_contents, path)) in queries.iter().enumerate() {
        let response = client.gql::<Value>(query_contents.to_owned()).send();
        let errors = dot_get_opt!(response, "errors", Vec::<serde_json::Value>).unwrap_or_default();
        assert!(errors.is_empty(), "Error response: {errors:?}");
        let value = dot_get_opt!(response, path, serde_json::Value).unwrap_or_default();
        let snapshot_name = format!("query_mutation_resolver_{case_index}_{index}", index = index + 1);
        if let Some(value) = value.as_str() {
            insta::assert_snapshot!(snapshot_name, value);
        } else {
            insta::assert_json_snapshot!(snapshot_name, value);
        }
    }
}

#[apply(test_query_mutation_resolver)]
fn test_query_mutation_resolver_start(
    case_index: usize,
    schema: &str,
    resolver_files: &[(&str, &str)],
    queries: &[(&str, &str)],
) {
    let mut env = Environment::init();
    env.grafbase_init(ConfigType::GraphQL);
    env.write_schema(schema);
    for (file_name, file_contents) in resolver_files {
        env.write_resolver(file_name, file_contents);
    }

    env.grafbase_start();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(60, 300);

    // Run queries.
    for (index, (query_contents, path)) in queries.iter().enumerate() {
        let response = client.gql::<Value>(query_contents.to_owned()).send();
        let errors = dot_get_opt!(response, "errors", Vec::<serde_json::Value>).unwrap_or_default();
        assert!(errors.is_empty(), "Error response: {errors:?}");
        let value = dot_get_opt!(response, path, serde_json::Value).unwrap_or_default();
        let snapshot_name = format!("query_mutation_resolver_start_{case_index}_{index}", index = index + 1);
        if let Some(value) = value.as_str() {
            insta::assert_snapshot!(snapshot_name, value);
        } else {
            insta::assert_json_snapshot!(snapshot_name, value);
        }
    }
}

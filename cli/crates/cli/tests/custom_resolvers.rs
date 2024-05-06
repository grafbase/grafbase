#![allow(unused_crate_dependencies)]
#![allow(clippy::too_many_lines)]
mod utils;

use backend::project::GraphType;
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
        type Query {
            post: Post! @resolver(name: "post")
        }

        type Post {
            title: String!
            text: String! @resolver(name: "return-text")
        }
    "#,
    &[
        ("post.js", r"
            export default function Resolver(parent, args, context, info) {
                return {
                    title: 'Hello'
                }
            }
        "),
        ("return-text.js", r#"
            export default function Resolver(parent, args, context, info) {
                return "Lorem ipsum dolor sit amet";
            }
        "#)
    ],
    &[
        ("{ post { text } }", "data.post.text")
    ],
    None,
)]
#[case(
    2,
    r#"
        type Query {
            post: Post! @resolver(name: "post")
        }

        type Post {
            title: String!
            fetchResult: JSON! @resolver(name: "fetch-grafbase-graphql")
        }
    "#,
    &[
        ("post.js", r"
            export default function Resolver(parent, args, context, info) {
                return {
                    title: 'Hello'
                }
            }
        "),
        ("fetch-grafbase-graphql.js", r#"
            export default function Resolver(parent, args, context, info) {
                return fetch('https://api.grafbase.com/graphql', {
                    headers: {
                        'content-type': 'application/json'
                    },
                    method: 'POST',
                    body: JSON.stringify({ query: '{ __typename }' })
                });
            }
        "#)
    ],
    &[
        ("{ post { fetchResult }  }", "data.post.fetchResult")
    ],
    None,
)]
#[case(
    3,
    r#"
        type Query {
            post: Post! @resolver(name: "post")
        }

        type Post {
            title: String!
            variable(name: String!): String @resolver(name: "return-env-variable")
        }
    "#,
    &[
        ("post.js", r"
            export default function Resolver(parent, args, context, info) {
                return {
                    title: 'Hello'
                }
            }
        "),
        ("return-env-variable.js", r#"
            export default function Resolver(parent, args, context, info) {
                return process.env[args.name] || null;
            }
        "#)
    ],
    &[
        (
            r#"{ post { variable(name: "GRAFBASE_ENV") } }"#,
            "post.variable"
        ),
        (
            r#"{ post { variable(name: "MY_OWN_VARIABLE") } }"#,
            "post.variable"
        ),
    ],
    None,
)]
#[case(
    4,
    r#"
        type Query {
            post: Post! @resolver(name: "post")
        }

        type Post {
            title: String!
            variable: String! @resolver(name: "return-env-variable")
        }
    "#,
    &[
        ("post.js", r"
            export default function Resolver(parent, args, context, info) {
                return {
                    title: 'Hello'
                }
            }
        "),
        ("return-env-variable.js", r#"
            const value = process.env["MY_OWN_VARIABLE"];

            export default function Resolver(parent, args, context, info) {
                return value;
            }
        "#)
    ],
    &[
        (
            "{ post { variable } }",
            "post.variable"
        ),
    ],
    None,
)]
#[case(
    5,
    r#"
        type Query {
            post: Post! @resolver(name: "post")
        }

        type Post {
            title: String!
            object: JSON! @resolver(name: "nested/return-object")
        }
    "#,
    &[
        ("post.js", r"
            export default function Resolver(parent, args, context, info) {
                return {
                    title: 'Hello'
                }
            }
        "),
        ("nested/return-object.ts", r#"
            export default function Resolver(parent, args, context, info) {
                const returnValue: any = { a: 123, b: "Hello" };
                return returnValue;
            }
        "#)
    ],
    &[
        ("{ post { object } }", "post.object")
    ],
    None,
)]
#[case(
    6,
    r#"
        type Query {
            post: Post! @resolver(name: "post")
        }

        type Post {
            title: String!
            title2: String! @resolver(name: "return-title")
        }
    "#,
    &[
        ("post.js", r"
            export default function Resolver(parent, args, context, info) {
                return {
                    title: 'Hello'
                }
            }
        "),
        ("return-title.js", r"
            export default function Resolver(parent, args, context, info) {
                return parent.title;
            }
        ")
    ],
    &[
        ("{ post { title2 } }", "post.title2")
    ],
    None,
)]
#[case(
    7,
    r#"
        type Query {
            post: Post! @resolver(name: "post")
        }

        type Post{
            title: String!
            headerValue(name: String!): String @resolver(name: "return-header-value")
        }
    "#,
    &[
        ("post.js", r"
            export default function Resolver(parent, args, context, info) {
                return {
                    title: 'Hello World!'
                }
            }
        "),
        ("return-header-value.js", r"
            export default function Resolver(parent, args, context, info) {
                return context.request.headers[args.name];
            }
        ")
    ],
    &[
        ("{ post { headerValue(name: \"x-test-header\") } }", "post.headerValue")
    ],
    None,
)]
#[case(
    8,
    r#"
        type Query {
            post: Post! @resolver(name: "post")
        }

        type Post {
            title: String!
            isTitlePalindrome: Boolean! @resolver(name: "resolver")
        }
    "#,
    &[
        ("post.js", r"
            export default function Resolver(parent, args, context, info) {
                return {
                    title: 'Hello World!'
                }
            }
        "),
        ("resolver.js", r"
            const isPalindrome = require('is-palindrome');
            export default function Resolver(parent, args, context, info) {
                return isPalindrome(parent.title);
            }
        ")
    ],
    &[
        ("{ post { isTitlePalindrome } }", "post.isTitlePalindrome")
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
        type Query {
            post: Post! @resolver(name: "post")
        }

        type Post {
            title: String!
            isTitlePalindrome: Boolean! @resolver(name: "resolver")
        }
    "#,
    &[
        ("post.js", r"
            export default function Resolver(parent, args, context, info) {
                return {
                    title: 'Hello World!'
                }
            }
        "),
        ("resolver.js", r"
            const isPalindrome = require('is-palindrome');
            export default function Resolver(parent, args, context, info) {
                return isPalindrome(parent.title);
            }
        ")
    ],
    &[
        ("{ post { isTitlePalindrome } }", "post.isTitlePalindrome")
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
        type Query {
            post: Post! @resolver(name: "post")
        }

        type Post {
            title: String!
            isTitlePalindrome: Boolean! @resolver(name: "resolver")
        }
    "#,
    &[
        ("post.js", r"
            export default function Resolver(parent, args, context, info) {
                return {
                    title: 'Hello World!'
                }
            }
        "),
        ("resolver.js", r"
            const isPalindrome = require('is-palindrome');
            export default function Resolver(parent, args, context, info) {
                return isPalindrome(parent.title);
            }
        ")
    ],
    &[
        ("{ post { isTitlePalindrome } }", "post.isTitlePalindrome")
    ],
    Some((JavaScriptPackageManager::Yarn, r#"
        {
            "dependencies": {
                "is-palindrome": "^0.3.0"
            },
            "packageManager": "yarn@1.22.0"
        }
    "#))
)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn test_field_resolver(
    #[case] case_index: usize,
    #[case] schema: &str,
    #[case] resolvers: &[(&str, &str)],
    #[case] queries: &[(&str, &str)],
    #[case] package_json: Option<(JavaScriptPackageManager, &str)>,
    #[values(("./", "./"), ("./grafbase", "../"))] variant: (&str, &str),
) {
    let (subdirectory_path, package_json_path) = variant;
    let mut env = Environment::init_in_subdirectory(subdirectory_path);
    env.grafbase_init(GraphType::Standalone);
    std::fs::write(
        env.directory_path.join(subdirectory_path).join(".env"),
        "MY_OWN_VARIABLE=test_value",
    )
    .unwrap();
    env.write_schema(schema);
    for (name, contents) in resolvers {
        env.write_resolver(name, contents);
    }
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
    client.poll_endpoint(120, 250).await;

    // Run queries.
    for (index, (query_contents, path)) in queries.iter().enumerate() {
        let response = client
            .gql::<Value>(query_contents.to_owned())
            .header("x-test-header", "test-value")
            .send()
            .await;
        let errors = dot_get_opt!(response, "errors", Vec::<serde_json::Value>).unwrap_or_default();
        assert!(errors.is_empty(), "Error response: {errors:?}");
        let data = dot_get!(response, "data", serde_json::Value);
        let value = dot_get_opt!(data, path, serde_json::Value).unwrap_or_default();
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
#[ignore]
fn test_query_mutation_resolver(
    #[case] case_index: usize,
    #[case] schema: &str,
    #[case] resolver_files: &[(&str, &str)],
    #[case] queries: &[(&str, &str)],
) {
}

#[apply(test_query_mutation_resolver)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_query_mutation_resolver_dev(
    case_index: usize,
    schema: &str,
    resolver_files: &[(&str, &str)],
    queries: &[(&str, &str)],
) {
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Standalone);
    env.write_schema(schema);
    for (file_name, file_contents) in resolver_files {
        env.write_resolver(file_name, file_contents);
    }

    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(60, 300).await;

    // Run queries.
    for (index, (query_contents, path)) in queries.iter().enumerate() {
        let response = client.gql::<Value>(query_contents.to_owned()).send().await;
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
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_query_mutation_resolver_start(
    case_index: usize,
    schema: &str,
    resolver_files: &[(&str, &str)],
    queries: &[(&str, &str)],
) {
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Standalone);
    env.write_schema(schema);
    for (file_name, file_contents) in resolver_files {
        env.write_resolver(file_name, file_contents);
    }

    env.grafbase_start();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(60, 300).await;

    // Run queries.
    for (index, (query_contents, path)) in queries.iter().enumerate() {
        let response = client.gql::<Value>(*query_contents).send().await;
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

#[test]
fn jwt_claims_in_custom_resolver_context() {
    let schema = r#"
        extend type Query {
            printClaims: String @resolver(name: "printClaims")
        }
    "#;
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Standalone);
    env.write_schema(schema);

    let resolver = r#"
        export default function(parent, args, ctx) {
            return `JWT claims: JSON.stringify(ctx.claims)``
        }
    "#;

    env.write_resolver("resolvers/printClaims.js", resolver);

    env.grafbase_start();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(60, 300).await;

    let response = client.gql::<Value>("{ printClaims }").send().await;

    let errors = dot_get_opt!(response, "errors", Vec::<serde_json::Value>).unwrap_or_default();
    assert!(errors.is_empty(), "Error response: {errors:?}");
    let value = dot_get_opt!(response, path, serde_json::Value).unwrap_or_default();

    assert_eq!(value, serde_json::json!("hi there"));
}

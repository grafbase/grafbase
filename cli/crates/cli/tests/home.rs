#![allow(unused_crate_dependencies)]
mod utils;

use backend::project::ConfigType;
use std::path::PathBuf;
use utils::environment::Environment;

#[rstest::rstest]
#[case(PathBuf::from("./temp"))]
#[case(dirs::home_dir().unwrap())]
fn flag(#[case] case_path: PathBuf) {
    let mut env = Environment::init().with_home(PathBuf::from(&case_path));
    env.write_schema(
        r#"
        type Post @model {
            title: String @resolver(name: "return-title")
        }
        "#,
    );
    env.write_resolver(
        "return-title.js",
        r#"
        export default function Resolver({ parent, args, context, info }) {
            return "title";
        }
        "#,
    );
    env.grafbase_dev();
    let client = env.create_client();
    client.poll_endpoint(30, 300);
}

#[rstest::rstest]
#[case(PathBuf::from("./temp"))]
#[case(dirs::home_dir().unwrap())]
fn env_var(#[case] case_path: PathBuf) {
    std::env::set_var("GRAFBASE_HOME", case_path.as_os_str());

    let mut env = Environment::init();
    env.grafbase_init(ConfigType::GraphQL);
    env.write_schema(
        r#"
        type Post @model {
            title: String @resolver(name: "return-title")
        }
        "#,
    );
    env.write_resolver(
        "return-title.js",
        r#"
        export default function Resolver({ parent, args, context, info }) {
            return "title";
        }
        "#,
    );
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);
}

#[rstest::rstest]
#[case(PathBuf::from("./temp"))]
#[case(dirs::home_dir().unwrap())]
#[cfg(not(target_os = "windows"))]
fn ts_config_flag(#[case] case_path: PathBuf) {
    let mut env = Environment::init().with_home(PathBuf::from(&case_path));
    env.set_typescript_config(include_str!("config/default.ts"));
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    let response = client
        .gql::<serde_json::Value>(
            r#"
        query {
            userCollection(first: 100) {
                edges {
                    node {
                        id
                    }
                }
            }
        }
    "#,
        )
        .send();
    assert_eq!(
        response,
        serde_json::json!({
            "data": {
                "userCollection": {
                    "edges": []
                }
            }
        })
    );
}

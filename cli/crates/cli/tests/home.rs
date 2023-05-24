mod utils;

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
    env.grafbase_init();
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
fn ts_config_flag(#[case] case_path: PathBuf) {
    let mut env = Environment::init().with_home(PathBuf::from(&case_path));
    env.set_typescript_config(include_str!("config/default.ts"));
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);
}

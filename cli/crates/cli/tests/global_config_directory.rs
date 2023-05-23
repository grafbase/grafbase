mod utils;

use std::path::PathBuf;
use utils::environment::Environment;

#[rstest::rstest]
#[case(PathBuf::from("./temp"))]
#[case(dirs::home_dir().unwrap())]
fn flag(#[case] case_path: PathBuf) {
    let mut env = Environment::init().with_global_config_directory(PathBuf::from(&case_path));
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
    std::env::set_var("GRAFBASE_GLOBAL_CONFIG_DIRECTORY", case_path.as_os_str());

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
    let mut env = Environment::init().with_global_config_directory(PathBuf::from(&case_path));
    env.prepare_ts_config_dependencies();
    env.write_ts_config(
        r#"
        import { config, g } from '@grafbase/sdk'

        const address = g.type('Address', {
            street: g.string().optional()
        })

        g.model('User', {
            name: g.string(),
            address: g.ref(address).optional()
        })

        export default config({ schema: g })
        "#,
    );
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);
}

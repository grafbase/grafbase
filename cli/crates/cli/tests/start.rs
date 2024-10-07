#![allow(unused_crate_dependencies)]
mod utils;

#[cfg(not(target_os = "windows"))]
use utils::environment::Environment;

#[ctor::ctor]
fn setup_rustls() {
    rustls::crypto::ring::default_provider().install_default().unwrap();
}

#[cfg(not(target_os = "windows"))]
#[rstest::rstest]
#[case(true)]
#[case(false)]
#[tokio::test]
async fn start_with_ts_config(#[case] module: bool) {
    let mut env = Environment::init();
    if module {
        env.prepare_ts_config_dependencies_module()
    } else {
        env.prepare_ts_config_dependencies()
    }

    env.set_typescript_config(indoc::indoc! { r#"
        import { config, g } from '@grafbase/sdk'

        g.query('hello', {
            args: { name: g.string().optional() },
            returns: g.string(),
            resolver: 'hello',
        })

        export default config({ schema: g })
    "#});
    env.write_resolver(
        "hello.js",
        indoc::indoc! {
            r#"
            export default function Resolver(_, {name}) {
                return `Hello ${name}`;
            }
            "#
        },
    );

    env.grafbase_dev();
    let client = env.create_async_client().with_api_key();
    client.poll_endpoint(30, 300).await;

    let response = client
        .gql::<serde_json::Value>(r#"query { hello(name: "there") }"#)
        .await;

    assert_eq!(
        response,
        serde_json::json!({
            "data": {
                "hello": "Hello there"
            }
        })
    );
}

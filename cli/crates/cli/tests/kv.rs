#![allow(unused_crate_dependencies)]
mod utils;

use backend::project::ConfigType;
use serde_json::Value;
use utils::environment::Environment;

#[rstest::rstest]
#[case(true)]
#[case(false)]
fn test_kv_integration(#[case] enabled: bool) {
    // prepare
    let mut env = Environment::init();
    env.grafbase_init(ConfigType::GraphQL);
    env.write_schema(format!(
        r#"
                extend schema @experimental(kv: {enabled})

                extend type Query {{
                    hello: String! @resolver(name: "test")
                }}
            "#
    ));
    env.write_resolver(
        "test.js",
        r#"
        export default async function Resolver(_, __, { kv }) {
            const kvKey = "test";

            let { value } = await kv.get(kvKey);
            if (value === null) {
                console.info(`Key ${kvKey} doesn't exist in KV. Creating ...`);
                await kv.set(kvKey, "hello kv!");
            }

            let { value: kv_value } = await kv.get(kvKey);

            return kv_value;
        }
    "#,
    );

    env.grafbase_dev();
    let client = env
        .create_client_with_options(utils::client::ClientOptionsBuilder::default().http_timeout(60).build())
        .with_api_key();
    client.poll_endpoint(120, 250);

    // act
    let response = client.gql::<Value>("query { hello }").send();

    // asert
    if enabled {
        assert_eq!(dot_get!(response, "data.hello", String), "hello kv!");
    } else {
        assert_eq!(dot_get!(response, "errors.0.message", String), "Invocation failed");
    }
}

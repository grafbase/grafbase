#![allow(unused_crate_dependencies)]
mod utils;

use utils::environment::Environment;

#[test]
#[cfg(not(target_os = "windows"))]
fn federation_start() {
    let mut env = Environment::init();
    env.set_typescript_config(
        r"
        import { config, graph } from '@grafbase/sdk'

        export default config({
          graph: graph.Federated(),
        })
        ",
    );
    env.grafbase_start();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    let response = client
        .gql::<serde_json::Value>(
            r"
        query {
          __schema {
            types {
              name
            }
          }
        }
    ",
        )
        .send();
    insta::assert_json_snapshot!(response, @r###"
    {
      "data": null,
      "errors": [
        {
          "message": "there are no subgraphs registered currently"
        }
      ]
    }
    "###);
}

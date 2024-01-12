#![allow(unused_crate_dependencies, unused_imports)]
mod utils;

use backend::project::GraphType;
use utils::environment::Environment;

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn federation_start() {
    use duct::cmd;

    let mut env = Environment::init();
    let output = env.grafbase_init_output(GraphType::Federated);
    assert!(output.status.success());

    let output = cmd!("npm", "install").dir(&env.directory_path).run().unwrap();
    assert!(output.status.success());

    env.grafbase_start();
    let client = env.create_client();
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
      "errors": [
        {
          "message": "there are no subgraphs registered currently"
        }
      ]
    }
    "###);
}

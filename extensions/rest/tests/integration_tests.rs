use grafbase_sdk::test::{DynamicSchema, ExtensionOnlySubgraph, TestConfigBuilder, TestRunner};
use indoc::{formatdoc, indoc};
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

const CLI_PATH: &str = "../../target/debug/grafbase";
const GATEWAY_PATH: &str = "../../target/debug/grafbase-gateway";

fn subgraph(rest_endpoint: &str) -> ExtensionOnlySubgraph {
    let extension_path = std::env::current_dir().unwrap().join("build");
    let path_str = format!("file://{}", extension_path.display());

    let schema = formatdoc! {r#"
        extend schema
          @link(url: "https://specs.apollo.dev/federation/v2.0", import: ["@key", "@shareable"])
          @link(url: "{path_str}", import: ["@restEndpoint", "@rest"])

        @restEndpoint(
          name: "endpoint",
          http: {{
            baseURL: "{rest_endpoint}"
          }}
        )

        type Query {{
          users: [User!]! @rest(
            endpoint: "endpoint",
            http: {{
              method: GET,
              path: "/users"
            }},
            selection: "[.[] | {{ id, name, age }}]"
          )
        }}

        type User {{
          id: ID!
          name: String!
          age: Int!
        }}
    "#};

    DynamicSchema::builder(schema)
        .into_extension_only_subgraph("test", &extension_path)
        .unwrap()
}

async fn mock_server(listen_path: &str, template: ResponseTemplate) -> MockServer {
    let mock_server = MockServer::builder().start().await;

    Mock::given(method("GET"))
        .and(path(listen_path))
        .respond_with(template)
        .mount(&mock_server)
        .await;

    mock_server
}

#[tokio::test]
async fn get_all_fields() {
    let response_body = json!([
        {
            "id": "1",
            "name": "John Doe",
            "age": 30,
            "nonimportant": 2,
        },
        {
            "id": "2",
            "name": "Jane Doe",
            "age": 25,
            "nonimportant": 3,
        }
    ]);

    let template = ResponseTemplate::new(200).set_body_json(response_body);
    let mock_server = mock_server("/users", template).await;
    let subgraph = subgraph(&mock_server.uri());

    let config = TestConfigBuilder::new()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph)
        .enable_networking()
        .build("")
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let query = indoc! {r#"
        query {
          users {
            id
            name
            age
          }
        }
    "#};

    let result: serde_json::Value = runner.graphql_query(query).send().await.unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "users": [
          {
            "id": "1",
            "name": "John Doe",
            "age": 30
          },
          {
            "id": "2",
            "name": "Jane Doe",
            "age": 25
          }
        ]
      }
    }
    "#);
}

#[tokio::test]
async fn get_some_fields() {
    let response_body = json!([
        {
            "id": "1",
            "name": "John Doe",
            "age": 30
        },
        {
            "id": "2",
            "name": "Jane Doe",
            "age": 25
        }
    ]);

    let template = ResponseTemplate::new(200).set_body_json(response_body);
    let mock_server = mock_server("/users", template).await;
    let subgraph = subgraph(&mock_server.uri());

    let config = TestConfigBuilder::new()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph)
        .enable_networking()
        .build("")
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let query = indoc! {r#"
        query {
          users {
            id
          }
        }
    "#};

    let result: serde_json::Value = runner.graphql_query(query).send().await.unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "users": [
          {
            "id": "1"
          },
          {
            "id": "2"
          }
        ]
      }
    }
    "#);
}

#[tokio::test]
async fn faulty_response() {
    let response_body = json!([
        {
            "foo": "1",
            "bar": "John Doe",
            "lol": 30
        }
    ]);

    let template = ResponseTemplate::new(200).set_body_json(response_body);
    let mock_server = mock_server("/users", template).await;
    let subgraph = subgraph(&mock_server.uri());

    let config = TestConfigBuilder::new()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph)
        .enable_networking()
        .build("")
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let query = indoc! {r#"
        query {
          users {
            id
          }
        }
    "#};

    let result: serde_json::Value = runner.graphql_query(query).send().await.unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "data": null,
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 3,
              "column": 5
            }
          ],
          "path": [
            "users",
            0,
            "id"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[tokio::test]
async fn internal_server_error() {
    let template = ResponseTemplate::new(500);
    let mock_server = mock_server("/users", template).await;
    let subgraph = subgraph(&mock_server.uri());

    let config = TestConfigBuilder::new()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph)
        .enable_networking()
        .build("")
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let query = indoc! {r#"
        query {
          users {
            id
          }
        }
    "#};

    let result: serde_json::Value = runner.graphql_query(query).send().await.unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "data": null,
      "errors": [
        {
          "message": "HTTP request failed with status: 500 Internal Server Error",
          "extensions": {
            "code": "INTERNAL_SERVER_ERROR"
          }
        }
      ]
    }
    "#);
}

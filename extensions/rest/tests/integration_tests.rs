use grafbase_sdk::test::{DynamicSchema, ExtensionOnlySubgraph, TestConfig, TestRunner};
use indoc::{formatdoc, indoc};
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, header, method, path},
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
          baseURL: "{rest_endpoint}"
        )

        type Query {{
          users: [User!]! @rest(
            endpoint: "endpoint",
            method: GET,
            path: "/users"
            selection: "[.[] | {{ id, name, age }}]"
          )

          user(id: Int!): User @rest(
            endpoint: "endpoint",
            method: GET,
            path: "/users/{{{{ args.id }}}}"
            selection: "{{ id, name, age }}"
          )
        }}

        type Mutation {{
          createUser(input: UserInput!): User! @rest(
            endpoint: "endpoint",
            method: POST,
            path: "/users"
            selection: "{{ id, name, age }}"
          )

          createStaticUser: User! @rest(
            endpoint: "endpoint",
            method: POST,
            path: "/users"
            body: {{ static: {{ name: "John Doe", age: 30 }} }}
            selection: "{{ id, name, age }}"
          )

          updateUser(id: Int!, input: UserInput!): User! @rest(
            endpoint: "endpoint",
            method: PUT,
            path: "/users/{{{{ args.id }}}}"
            selection: "{{ id, name, age }}"
          )

          deleteUser(id: Int!): User! @rest(
            endpoint: "endpoint",
            method: DELETE,
            path: "/users/{{{{ args.id }}}}"
            selection: "{{ id, name, age }}"
          )
        }}

        type User {{
          id: ID!
          name: String!
          age: Int
        }}

        input UserInput {{
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
async fn get_many() {
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

    let config = TestConfig::builder()
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
async fn get_one() {
    let response_body = json!({
        "id": "1",
        "name": "John Doe",
        "age": 30,
    });

    let template = ResponseTemplate::new(200).set_body_json(response_body);
    let mock_server = mock_server("/users/1", template).await;
    let subgraph = subgraph(&mock_server.uri());

    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph)
        .enable_networking()
        .build("")
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let query = indoc! {r#"
        query {
          user(id: 1) {
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
        "user": {
          "id": "1",
          "name": "John Doe",
          "age": 30
        }
      }
    }
    "#);
}

#[tokio::test]
async fn get_one_missing() {
    let response_body = json!(null);

    let template = ResponseTemplate::new(200).set_body_json(response_body);
    let mock_server = mock_server("/users/1", template).await;
    let subgraph = subgraph(&mock_server.uri());

    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph)
        .enable_networking()
        .build("")
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let query = indoc! {r#"
        query {
          user(id: 1) {
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
        "user": null
      }
    }
    "#);
}

#[tokio::test]
async fn get_one_nested_null() {
    let response_body = json!({
        "id": "1",
        "name": "John Doe",
        "age": null,
    });

    let template = ResponseTemplate::new(200).set_body_json(response_body);
    let mock_server = mock_server("/users/1", template).await;
    let subgraph = subgraph(&mock_server.uri());

    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph)
        .enable_networking()
        .build("")
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let query = indoc! {r#"
        query {
          user(id: 1) {
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
        "user": {
          "id": "1",
          "name": "John Doe",
          "age": null
        }
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

    let config = TestConfig::builder()
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

    let config = TestConfig::builder()
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

    let config = TestConfig::builder()
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
            "code": "EXTENSION_ERROR"
          }
        }
      ]
    }
    "#);
}

#[tokio::test]
async fn with_bad_jq() {
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
    let extension_path = std::env::current_dir().unwrap().join("build");
    let path_str = format!("file://{}", extension_path.display());
    let rest_endpoint = mock_server.uri();

    let schema = formatdoc! {r#"
        extend schema
          @link(url: "https://specs.apollo.dev/federation/v2.0", import: ["@key", "@shareable"])
          @link(url: "{path_str}", import: ["@restEndpoint", "@rest"])

        @restEndpoint(
          name: "endpoint",
          baseURL: "{rest_endpoint}"
        )

        type Query {{
          users: [User!]! @rest(
            endpoint: "endpoint",
            method: GET,
            path: "/users"
            selection: "\\||\\"
          )
        }}

        type User {{
          id: ID!
          name: String!
          age: Int!
        }}
    "#};

    let subgraph = DynamicSchema::builder(schema)
        .into_extension_only_subgraph("test", &extension_path)
        .unwrap();

    let config = TestConfig::builder()
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
      "data": null,
      "errors": [
        {
          "message": "Error selecting result value: The selection is not valid jq syntax: `\\||\\`",
          "extensions": {
            "code": "EXTENSION_ERROR"
          }
        }
      ]
    }
    "#);
}

#[tokio::test]
async fn with_path_in_the_endpoint() {
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
    let mock_server = mock_server("/admin/users", template).await;
    let extension_path = std::env::current_dir().unwrap().join("build");
    let path_str = format!("file://{}", extension_path.display());
    let rest_endpoint = mock_server.uri();

    let schema = formatdoc! {r#"
        extend schema
          @link(url: "https://specs.apollo.dev/federation/v2.0", import: ["@key", "@shareable"])
          @link(url: "{path_str}", import: ["@restEndpoint", "@rest"])

        @restEndpoint(
          name: "endpoint",
          baseURL: "{rest_endpoint}/admin"
        )

        type Query {{
          users: [User!]! @rest(
            endpoint: "endpoint",
            method: GET,
            path: "/users"
            selection: "[.[] | {{ id, name, age }}]"
          )
        }}

        type User {{
          id: ID!
          name: String!
          age: Int!
        }}
    "#};

    let subgraph = DynamicSchema::builder(schema)
        .into_extension_only_subgraph("test", &extension_path)
        .unwrap();

    let config = TestConfig::builder()
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
async fn update_user() {
    let request_body = json!({
        "name": "John Doe",
        "age": 30,
    });

    let response_body = json!({
        "id": "1",
        "name": "John Doe",
        "age": 30,
    });

    let template = ResponseTemplate::new(201).set_body_json(response_body);
    let mock_server = MockServer::builder().start().await;

    Mock::given(method("PUT"))
        .and(path("/users/1"))
        .and(body_json(request_body))
        .and(header("Content-Type", "application/json"))
        .respond_with(template)
        .mount(&mock_server)
        .await;

    let subgraph = subgraph(&mock_server.uri());

    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph)
        .enable_networking()
        .build("")
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let mutation = indoc! {r#"
        mutation {
          updateUser(id: 1, input: { name: "John Doe", age: 30 }) {
            id
            name
            age
          }
        }
    "#};

    let result: serde_json::Value = runner.graphql_query(mutation).send().await.unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "updateUser": {
          "id": "1",
          "name": "John Doe",
          "age": 30
        }
      }
    }
    "#);
}

#[tokio::test]
async fn delete_user() {
    let response_body = json!({
        "id": "1",
        "name": "John Doe",
        "age": 30,
    });

    let template = ResponseTemplate::new(201).set_body_json(response_body);
    let mock_server = MockServer::builder().start().await;

    Mock::given(method("DELETE"))
        .and(path("/users/1"))
        .respond_with(template)
        .mount(&mock_server)
        .await;

    let subgraph = subgraph(&mock_server.uri());

    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph)
        .enable_networking()
        .build("")
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let mutation = indoc! {r#"
        mutation {
          deleteUser(id: 1) {
            id
            name
            age
          }
        }
    "#};

    let result: serde_json::Value = runner.graphql_query(mutation).send().await.unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "deleteUser": {
          "id": "1",
          "name": "John Doe",
          "age": 30
        }
      }
    }
    "#);
}

#[tokio::test]
async fn dynamic_post() {
    let request_body = json!({
        "name": "John Doe",
        "age": 30,
    });

    let response_body = json!({
        "id": "1",
        "name": "John Doe",
        "age": 30,
    });

    let template = ResponseTemplate::new(201).set_body_json(response_body);
    let mock_server = MockServer::builder().start().await;

    Mock::given(method("POST"))
        .and(path("/users"))
        .and(body_json(request_body))
        .and(header("Content-Type", "application/json"))
        .respond_with(template)
        .mount(&mock_server)
        .await;

    let subgraph = subgraph(&mock_server.uri());

    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph)
        .enable_networking()
        .build("")
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let mutation = indoc! {r#"
        mutation {
          createUser(input: { name: "John Doe", age: 30 }) {
            id
            name
            age
          }
        }
    "#};

    let result: serde_json::Value = runner.graphql_query(mutation).send().await.unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "createUser": {
          "id": "1",
          "name": "John Doe",
          "age": 30
        }
      }
    }
    "#);
}

#[tokio::test]
async fn static_post() {
    let request_body = json!({
        "name": "John Doe",
        "age": 30,
    });

    let response_body = json!({
        "id": "1",
        "name": "John Doe",
        "age": 30,
    });

    let template = ResponseTemplate::new(201).set_body_json(response_body);
    let mock_server = MockServer::builder().start().await;

    Mock::given(method("POST"))
        .and(path("/users"))
        .and(body_json(request_body))
        .and(header("Content-Type", "application/json"))
        .respond_with(template)
        .mount(&mock_server)
        .await;

    let subgraph = subgraph(&mock_server.uri());

    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph)
        .enable_networking()
        .build("")
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let mutation = indoc! {r#"
        mutation {
          createStaticUser {
            id
            name
            age
          }
        }
    "#};

    let result: serde_json::Value = runner.graphql_query(mutation).send().await.unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "createStaticUser": {
          "id": "1",
          "name": "John Doe",
          "age": 30
        }
      }
    }
    "#);
}

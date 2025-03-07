use grafbase_sdk::test::{DynamicSchema, TestConfig, TestRunner};
use wiremock::matchers;

const CLI_PATH: &str = "../../target/debug/grafbase";
const GATEWAY_PATH: &str = "../../target/debug/grafbase-gateway";

#[tokio::test]
async fn test_basic_responses() {
    let extension_path = std::env::current_dir().unwrap().join("build");
    let extension_path_str = format!("file://{}", extension_path.display());

    let subgraph = DynamicSchema::builder(format!(
        r#"
        extend schema
          @link(url: "https://specs.apollo.dev/federation/v2.7")
          @link(url: "{extension_path_str}", import: ["@snowflakeQuery"])

        scalar JSON

        type Query {{
          hi(params: [JSON!]!): [[JSON!]!] @snowflakeQuery(sql: "SELECT ?", bindings: "{{{{ args.params }}}}")
          users(params: [JSON!]!): [[JSON!]!]
            @snowflakeQuery(sql: "SELECT * FROM CUSTOMER LIMIT ?;", bindings: "{{{{ args.params }}}}")
        }}
        "#
    ))
    .into_extension_only_subgraph("test-subgraph", &extension_path)
    .unwrap();

    let mock_server = wiremock::MockServer::start().await;
    let mock_server_url = mock_server.address();

    let test_rsa_private_key = include_str!("./test_rsa_key.p8");
    let test_rsa_public_key = include_str!("./test_rsa_key.pub");

    let config = format! {r#"
        [extensions.snowflake.config]
        account = "cywwwdp-qv94952"
        user = "tomhoule"

        snowflake_api_url_override = "http://{mock_server_url}"

        warehouse = "COMPUTE_WH"
        database = "SNOWFLAKE_SAMPLE_DATA"
        schema = "TPCH_SF1"
        # role = ""

        [extensions.snowflake.config.authentication.key_pair_jwt]
        public_key = """
        {test_rsa_public_key}
        """
        private_key = """
        {test_rsa_private_key}
        """
    "#};

    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph)
        .enable_networking()
        .build(config)
        .unwrap();

    // A runner for building the extension, and executing the Grafbase Gateway together
    // with the subgraphs. The runner composes all subgraphs into a federated schema.
    let runner = TestRunner::new(config).await.unwrap();

    wiremock::Mock::given(matchers::method("POST"))
        .and(matchers::path("/api/v2/statements"))
        .and(matchers::body_partial_json(serde_json::json!({
            "statement": "SELECT ?",
        })))
        .respond_with(wiremock::ResponseTemplate::new(200)
        .set_body_raw(
        r#"
{
"resultSetMetaData" : {
 "numRows" : 1,
 "format" : "jsonv2",
 "partitionInfo" : [ {
   "rowCount" : 1,
   "uncompressedSize" : 8
 } ],
 "rowType" : [ {
   "name" : "?",
   "database" : "",
   "schema" : "",
   "table" : "",
   "nullable" : false,
   "length" : null,
   "type" : "fixed",
   "scale" : 0,
   "precision" : 4,
   "byteLength" : null,
   "collation" : null
 } ]
},
"data" : [ ["9999"] ],
"code" : "090001",
"statementStatusUrl" : "/api/v2/statements/01bad80f-0000-4392-0000-3c790002f19e?requestId=7c765ba6-f6e3-4407-bb44-206bf63ddd96",
"requestId" : "7c765ba6-f6e3-4407-bb44-206bf63ddd96",
"sqlState" : "00000",
"statementHandle" : "01bad80f-0000-4392-0000-3c790002f19e",
"message" : "Statement executed successfully.",
"createdOn" : 1741333409656
}
"#, "application/json"
    )).mount(&mock_server).await;

    let result: serde_json::Value = runner
        .graphql_query(r#"query { hi(params: [9999]) }"#)
        .send()
        .await
        .unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "hi": [
          [
            "9999"
          ]
        ]
      }
    }
    "#);

    wiremock::Mock::given(matchers::method("POST"))
        .and(matchers::path("/api/v2/statements"))
        .and(matchers::body_json(serde_json::json!({
            "statement": "SELECT * FROM CUSTOMER LIMIT ?;",
            "bindings": {
                "1":  {
                    "type": "TEXT",
                    "value": "abcd",
                },
            },
            "database": "SNOWFLAKE_SAMPLE_DATA",
            "schema": "TPCH_SF1",
            "warehouse": "COMPUTE_WH",
            "role": null,
        })))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_raw(
            r#"
{
"code" : "002010",
"message" : "SQL compilation error:\nInvalid row count '?' in limit clause",
"sqlState" : "2201W",
"statementHandle" : "01bad820-0000-43c9-0000-3c790003200e"
}
"#,
            "application/json",
        ))
        .mount(&mock_server)
        .await;

    let result: serde_json::Value = runner
        .graphql_query(r#"query { users(params: ["abcd"]) }"#)
        .send()
        .await
        .unwrap();

    // panic!(
    //     "{:#?}",

    //     mock_server.received_requests().await.unwrap()[1]
    //         .body_json::<serde_json::Value>()
    //         .unwrap()
    // );

    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "users": null
      },
      "errors": [
        {
          "message": "No data returned from Snowflake query. SQL State: 2201W, Code: 002010. Message: SQL compilation error:\nInvalid row count '?' in limit clause",
          "locations": [
            {
              "line": 1,
              "column": 9
            }
          ],
          "extensions": {
            "code": "EXTENSION_ERROR"
          }
        }
      ]
    }
    "#);
}

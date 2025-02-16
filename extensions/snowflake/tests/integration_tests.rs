use grafbase_sdk::test::{DynamicSchema, TestConfig, TestRunner};
use indoc::indoc;

#[tokio::test]
async fn test_snowflake_query() {
    let subgraph = DynamicSchema::builder(
        r#"
        type Query {
            getUsers: String! @snowflakeQuery(
                sql: "SELECT * FROM users"
            )
        }
        "#,
    )
    .into_subgraph("test")
    .unwrap();

    let config = indoc! {r#"
        [extensions.snowflake]
        [extensions.snowflake.config]
        account = "your_account"
        username = "your_username"
        password = "your_password"
        database = "your_database"
        warehouse = "your_warehouse"
        role = "your_role"
    "#};

    let config = TestConfig::builder()
        .with_subgraph(subgraph)
        .enable_networking()
        .build(config)
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let result: serde_json::Value = runner
        .graphql_query(r#"query { getUsers }"#)
        .send()
        .await
        .unwrap();

    insta::assert_json_snapshot!(result);
} 
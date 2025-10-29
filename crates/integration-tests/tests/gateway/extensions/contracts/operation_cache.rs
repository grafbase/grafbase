use std::collections::HashSet;

use graphql_mocks::dynamic::DynamicSchemaBuilder;
use integration_tests::{gateway::Gateway, runtime};

const CONTRACT_KEY: &str = r#"{"excludedTags":["internal"]}"#;
const SCHEMA_SDL: &str = r#"
extend schema @link(url: "contracts-19", import: ["@tag"])

type Query {
  public: ID! @tag(name: "public")
  private: ID! @tag(name: "internal")
}
"#;
const QUERY: &str = r#"query Test { public }"#;

#[test]
fn contract_plans_use_distinct_operation_cache_keys() {
    runtime().block_on(async move {
        let user_subgraph = DynamicSchemaBuilder::new(SCHEMA_SDL)
            .with_resolver("Query", "public", "public-value")
            .with_resolver("Query", "private", "secret-value")
            .into_subgraph("user");

        let gateway = Gateway::builder()
            .with_subgraph(user_subgraph)
            .with_extension("contracts-19")
            .with_extension("hooks-19")
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                "#,
            )
            .build()
            .await;

        let initial_count = gateway.operation_cache_keys().len();

        let base_response = gateway.post(QUERY).await;
        tokio::task::yield_now().await;
        assert!(
            base_response.errors().is_empty(),
            "expected base request to succeed, errors: {:?}",
            base_response.errors()
        );
        let base_keys: HashSet<_> = gateway.operation_cache_keys().into_iter().collect();
        assert!(
            base_keys.len() > initial_count,
            "expected the base request to populate the operation cache"
        );

        let contract_response = gateway.post(QUERY).header("contract-key", CONTRACT_KEY).await;
        tokio::task::yield_now().await;
        assert!(
            contract_response.errors().is_empty(),
            "expected contract request to succeed, errors: {:?}",
            contract_response.errors()
        );
        let contract_keys: HashSet<_> = gateway.operation_cache_keys().into_iter().collect();
        assert!(
            contract_keys.len() > base_keys.len(),
            "expected the contract request to insert a distinct plan into the operation cache; base keys: {:?}, contract keys: {:?}",
            base_keys,
            contract_keys
        );
    });
}

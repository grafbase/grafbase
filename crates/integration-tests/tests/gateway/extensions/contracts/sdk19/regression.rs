use std::time::Duration;

use graphql_mocks::dynamic::{DynamicSchema, Resolver, ResolverContext};
use rand::{Rng, distributions::Alphanumeric};
use serde_json::json;

use integration_tests::{gateway::Gateway, runtime};

const SDL: &str = r#"
extend schema @link(url: "contracts-19", import: ["@tag"])

type Query @tag(name: "public") {
    viewer: User
}

type User {
    id: ID! @tag(name: "public")
    secret: String! @tag(name: "secret")
}
"#;

const QUERY: &str = r#"
query Viewer {
    viewer {
        id
        secret
    }
}
"#;

#[test]
fn redis_operation_cache_serves_default_contract() {
    runtime().block_on(async move {
        let key_prefix = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect::<String>();

        let contract_key = json!({
            "includedTags": ["public"],
            "excludedTags": ["secret"]
        });

        let config = format!(
            r#"
        [graph]
        introspection = true
        [graph.contracts.cache]
        max_size = 2

        [operation_caching]
        enabled = true
        limit = 1000
        redis.url = "redis://localhost:6379"
        redis.key_prefix = "tests-{key_prefix}-"
        "#,
        );

        // Simulate the gateway successfully serving a contract before restarting.
        {
            let gateway = Gateway::builder()
                .with_subgraph(
                    DynamicSchema::builder(SDL)
                        .with_resolver("Query", "viewer", ViewerResolver)
                        .with_resolver("User", "id", "user-1")
                        .with_resolver("User", "secret", "classified")
                        .into_subgraph("users"),
                )
                .with_extension("contracts-19")
                .with_extension("hooks-19")
                .with_toml_config(&config)
                .build()
                .await;

            let contract_response = gateway
                .post(QUERY)
                .header("contract-key", serde_json::to_vec(&contract_key).unwrap())
                .await;
            assert!(
                !contract_response.errors().is_empty(),
                "contract request before restart should fail (secret field hidden): {contract_response}"
            );
        }

        // Warm the default contract (no feature tags).
        let gateway = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(SDL)
                    .with_resolver("Query", "viewer", ViewerResolver)
                    .with_resolver("User", "id", "user-1")
                    .with_resolver("User", "secret", "classified")
                    .into_subgraph("users"),
            )
            .with_extension("contracts-19")
            .with_extension("hooks-19")
            .with_toml_config(&config)
            .build()
            .await;

        let baseline = gateway.post(QUERY).await;
        assert!(
            baseline.errors().is_empty(),
            "default request should succeed after restart: {baseline}"
        );

        // Default schema should still expose the secret field.
        insta::assert_yaml_snapshot!(baseline, @r"
        data:
          viewer:
            id: user-1
            secret: classified
        ");

        assert_eq!(
            baseline["data"]["viewer"]["secret"], "classified",
            "default schema should still expose secret field"
        );

        // Give the background task time to persist the cached plan to Redis.
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Second request with the contract key that was working before the restart.
        let contract = gateway
            .post(QUERY)
            .header("contract-key", serde_json::to_vec(&contract_key).unwrap())
            .await;

        // contract request should fail (secret field hidden in contract schema)
        insta::assert_yaml_snapshot!(contract, @r#"
        errors:
          - message: "User does not have a field named 'secret'."
            locations:
              - line: 5
                column: 9
            extensions:
              code: OPERATION_VALIDATION_ERROR
        "#);
    });
}

#[derive(Clone, Copy)]
struct ViewerResolver;

impl Resolver for ViewerResolver {
    fn resolve(&mut self, _: ResolverContext<'_>) -> Option<serde_json::Value> {
        Some(json!({
            "__typename": "User",
            "id": "user-1",
            "secret": "classified"
        }))
    }
}

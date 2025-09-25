mod exclude;
mod include;
mod mixed_tags;
mod unreachable_types;

use integration_tests::{gateway::Gateway, runtime};

fn run(sdl: &str, key: &serde_json::Value) -> String {
    runtime().block_on(async move {
        Gateway::builder()
            .with_subgraph_sdl(
                "x",
                format!(
                    r#"
                    extend schema @link(url: "contracts-19", import: ["@tag"])
                    {sdl}
                    "#
                ),
            )
            .with_extension("contracts-19")
            .with_extension("hooks-19")
            .with_toml_config(
                r#"
            [graph]
            introspection = true
            "#,
            )
            .build()
            .await
            .introspect()
            .header("contract-key", serde_json::to_vec(key).unwrap())
            .await
    })
}

fn run_hide_unreachable_types(sdl: &str, key: &serde_json::Value) -> String {
    runtime().block_on(async move {
        Gateway::builder()
            .with_subgraph_sdl(
                "x",
                format!(
                    r#"
                    extend schema @link(url: "contracts-19", import: ["@tag"])
                    {sdl}
                    "#
                ),
            )
            .with_extension("contracts-19")
            .with_extension("hooks-19")
            .with_toml_config(
                r#"
            [graph]
            introspection = true

            [extensions.contracts-19.config]
            hide_unreachable_types = true
            "#,
            )
            .build()
            .await
            .introspect()
            .header("contract-key", serde_json::to_vec(key).unwrap())
            .await
    })
}

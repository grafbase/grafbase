use crate::{
    config::{build_config, Config},
    errors::ServerError,
};

#[tokio::main]
pub async fn dump_config(cli_version: String) -> Result<String, ServerError> {
    let env = crate::environment::variables().collect();

    let Config { registry, .. } = build_config(&env, None).await?;

    serde_json::to_string(&RegistryWithVersion { cli_version, registry }).map_err(ServerError::SchemaParserResultJson)
}

#[derive(serde::Serialize)]
struct RegistryWithVersion {
    cli_version: String,
    registry: engine::Registry,
}

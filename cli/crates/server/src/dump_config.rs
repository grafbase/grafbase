use crate::{
    config::{build_config, Config},
    errors::ServerError,
    servers::EnvironmentName,
};

#[tokio::main]
pub async fn dump_config(cli_version: String) -> Result<String, ServerError> {
    let env = crate::environment::variables(EnvironmentName::None).collect();

    let Config { registry, .. } = build_config(&env, None, EnvironmentName::None).await?;

    serde_json::to_string(&RegistryWithVersion { cli_version, registry }).map_err(ServerError::SchemaParserResultJson)
}

#[derive(serde::Serialize)]
struct RegistryWithVersion {
    cli_version: String,
    registry: engine::Registry,
}

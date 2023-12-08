use crate::{
    errors::ServerError,
    servers::{run_schema_parser, ParsingResponse},
};

#[tokio::main]
pub async fn dump_config(cli_version: String) -> Result<String, ServerError> {
    let env = crate::environment::variables().collect();

    let ParsingResponse {
        registry,
        detected_udfs: _,
        federated_graph_config: _,
    } = run_schema_parser(&env, None).await?;

    serde_json::to_string(&RegistryWithVersion { cli_version, registry }).map_err(ServerError::SchemaParserResultJson)
}

#[derive(serde::Serialize)]
struct RegistryWithVersion {
    cli_version: String,
    registry: engine::Registry,
}

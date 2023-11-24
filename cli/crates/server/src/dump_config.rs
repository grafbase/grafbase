use crate::{
    errors::ServerError,
    servers::{run_schema_parser, ParsingResponse},
};

#[tokio::main]
pub async fn dump_config(version: String) -> Result<String, ServerError> {
    let env = crate::environment::variables().collect();

    let ParsingResponse {
        mut registry,
        detected_udfs: _,
    } = run_schema_parser(&env, None).await?;

    registry.grafbase_cli_version = Some(version);

    serde_json::to_string(&registry).map_err(ServerError::SchemaParserResultJson)
}

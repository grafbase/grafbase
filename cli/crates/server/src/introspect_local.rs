use crate::{
    errors::ServerError,
    servers::{run_schema_parser, ParsingResponse},
};

pub enum IntrospectLocalOutput {
    Sdl(String),
    EmptyFederated,
}

#[tokio::main]
pub async fn introspect_local() -> Result<IntrospectLocalOutput, ServerError> {
    let env = crate::environment::variables().collect();

    let ParsingResponse {
        registry,
        detected_udfs: _,
        federated_graph_config,
    } = run_schema_parser(&env, None).await?;

    let is_federated = federated_graph_config.is_some();

    let rendered_sdl = registry.export_sdl(is_federated);

    if is_federated && rendered_sdl.is_empty() {
        Ok(IntrospectLocalOutput::EmptyFederated)
    } else {
        Ok(IntrospectLocalOutput::Sdl(rendered_sdl))
    }
}

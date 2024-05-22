use engine::registry::RegistrySdlExt;

use crate::{
    config::{build_config, Config},
    errors::ServerError,
    servers::EnvironmentName,
};

pub enum IntrospectLocalOutput {
    Sdl(String),
    EmptyFederated,
}

#[tokio::main]
pub async fn introspect_local() -> Result<IntrospectLocalOutput, ServerError> {
    let env = crate::environment::variables(EnvironmentName::None).collect();

    let Config {
        registry,
        federated_graph_config,
        ..
    } = build_config(&env, None, EnvironmentName::None).await?;

    let rendered_sdl = registry.export_sdl(registry.enable_federation);

    if federated_graph_config.is_some() && rendered_sdl.is_empty() {
        Ok(IntrospectLocalOutput::EmptyFederated)
    } else {
        Ok(IntrospectLocalOutput::Sdl(prettify(rendered_sdl)))
    }
}

fn prettify(graphql: String) -> String {
    match cynic_parser::parse_type_system_document(&graphql) {
        Ok(parsed) => parsed.to_sdl_pretty(),
        Err(_) => graphql,
    }
}

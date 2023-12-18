use crate::{
    config::{build_config, Config},
    errors::ServerError,
};

pub enum IntrospectLocalOutput {
    Sdl(String),
    EmptyFederated,
}

#[tokio::main]
pub async fn introspect_local() -> Result<IntrospectLocalOutput, ServerError> {
    let env = crate::environment::variables().collect();

    let Config {
        registry,
        federated_graph_config,
        ..
    } = build_config(&env, None).await?;

    let is_federated = federated_graph_config.is_some();

    let rendered_sdl = registry.export_sdl(is_federated);

    if is_federated && rendered_sdl.is_empty() {
        Ok(IntrospectLocalOutput::EmptyFederated)
    } else {
        Ok(IntrospectLocalOutput::Sdl(rendered_sdl))
    }
}

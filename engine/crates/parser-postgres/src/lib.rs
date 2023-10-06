use parser_sdl::Registry;
use postgres_types::{transport::Transport, Result};

mod introspection;
mod registry;

pub async fn introspect<T>(transport: &T, name: &str, namespaced: bool) -> Result<Registry>
where
    T: Transport + Sync,
{
    let database_definition = introspection::introspect(transport).await?;
    let registry = registry::generate(database_definition, name, namespaced);

    Ok(registry)
}

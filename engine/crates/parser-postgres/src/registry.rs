mod context;
mod queries;
mod types;

use context::InputContext;
use parser_sdl::Registry;
use postgres_types::database_definition::DatabaseDefinition;

use self::context::OutputContext;

pub(super) fn generate(database_definition: DatabaseDefinition, name: &str, namespaced: bool) -> Registry {
    let input_ctx = InputContext::new(database_definition, name, namespaced);
    let mut output_ctx = OutputContext::new(namespaced.then_some(name));

    types::generate(&input_ctx, &mut output_ctx);
    queries::generate(&input_ctx, &mut output_ctx);

    output_ctx.finalize(input_ctx.finalize(), name)
}

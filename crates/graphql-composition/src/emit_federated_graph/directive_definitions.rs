use super::*;

pub(super) fn emit_directive_definitions(ir: &CompositionIr, ctx: &mut Context<'_>) {
    ctx.out.directive_definitions = ir
        .directive_definitions
        .iter()
        .map(|definition| federated::DirectiveDefinition {
            namespace: None,
            name: definition.name,
            locations: definition.locations,
            arguments: definition.arguments,
            repeatable: definition.repeatable,
        })
        .collect();
}

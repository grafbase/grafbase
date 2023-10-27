use crate::{
    subgraphs::{DefinitionKind, DefinitionWalker},
    Context,
};

pub(crate) fn build_supergraph(ctx: &mut Context<'_>) {
    ctx.subgraphs
        .iter_definition_groups(|first, rest| match first.kind() {
            DefinitionKind::Object => merge_object_definitions(ctx, first, rest),
            _ => todo!(),
        });

    ctx.subgraphs.iter_field_groups(|first, rest| {
        if let Some(next) = rest.next() {
            ctx.diagnostics.push_fatal(format!(
                "The field `{}` on `{}` is defined in two subgraphs (`{}` and `{}`).",
                first.name_str(),
                first.parent_definition().name_str(),
                first.parent_definition().subgraph().name_str(),
                next.parent_definition().subgraph().name_str(),
            ));
        }

        ctx.supergraph.insert_field(
            first.parent_definition().name(),
            first.name(),
            first.type_name(),
        )
    });
}

fn merge_object_definitions<'a>(
    ctx: &mut Context<'_>,
    first: DefinitionWalker<'a>,
    mut rest: impl Iterator<Item = DefinitionWalker<'a>>,
) {
    let kind = first.kind();

    if let Some(incompatible) = rest.find(|definition| definition.kind() != kind) {
        let first_kind = first.kind();
        let second_kind = incompatible.kind();
        let name = first.name_str();
        let first_subgraph = first.subgraph().name_str();
        let second_subgraph = incompatible.subgraph().name_str();
        ctx.diagnostics.push_fatal(format!(
            "Cannot merge {first_kind:?} with {second_kind:?} (`{name}` in `{first_subgraph}` and `{second_subgraph}`)",
        ));
    }

    ctx.supergraph
        .insert_definition(first.name(), DefinitionKind::Object);
}

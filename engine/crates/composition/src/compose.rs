mod context;
mod enums;
mod input_object;
mod interface;
mod object;

pub(crate) use self::context::Context as ComposeContext;

use self::{context::Context, input_object::*};
use crate::subgraphs::{DefinitionKind, DefinitionWalker, FieldWalker, StringId};
use itertools::Itertools;

pub(crate) fn compose_subgraphs(ctx: &mut Context<'_>) {
    ctx.subgraphs.iter_definition_groups(|definitions| {
        let Some(first) = definitions.first() else {
            return;
        };

        match first.kind() {
            DefinitionKind::Object => merge_object_definitions(ctx, first, definitions),
            DefinitionKind::Union => merge_union_definitions(ctx, first, definitions),
            DefinitionKind::InputObject => merge_input_object_definitions(ctx, first, definitions),
            DefinitionKind::Interface => interface::merge_interface_definitions(ctx, first, definitions),
            DefinitionKind::Scalar => {
                ctx.insert_scalar(first.name());
            }
            DefinitionKind::Enum => enums::merge_enum_definitions(first, definitions, ctx),
        }
    });

    ctx.subgraphs
        .iter_field_groups(|fields| merge_field_definitions(fields, ctx));

    if !ctx.has_query_type() {
        ctx.diagnostics
            .push_fatal("The root `Query` object is not defined in any subgraph.".to_owned());
    }
}

fn merge_object_definitions<'a>(
    ctx: &mut Context<'_>,
    first: &DefinitionWalker<'a>,
    definitions: &[DefinitionWalker<'a>],
) {
    if let Some(incompatible) = definitions
        .iter()
        .find(|definition| definition.kind() != DefinitionKind::Object)
    {
        let first_kind = first.kind();
        let second_kind = incompatible.kind();
        let name = first.name().as_str();
        let first_subgraph = first.subgraph().name().as_str();
        let second_subgraph = incompatible.subgraph().name().as_str();
        ctx.diagnostics.push_fatal(format!(
            "Cannot merge {first_kind:?} with {second_kind:?} (`{name}` in `{first_subgraph}` and `{second_subgraph}`)",
        ));
    }

    let first_is_entity = first.is_entity();
    if definitions.iter().any(|object| object.is_entity() != first_is_entity) {
        let name = first.name().as_str();
        let (entity_subgraphs, non_entity_subgraphs) = definitions
            .iter()
            .partition::<Vec<DefinitionWalker<'_>>, _>(|definition| definition.is_entity());

        ctx.diagnostics.push_fatal(format!(
            "The `{name}` object is an entity in subgraphs {} but not in subgraphs {}.",
            entity_subgraphs
                .into_iter()
                .map(|d| d.subgraph().name().as_str())
                .join(", "),
            non_entity_subgraphs
                .into_iter()
                .map(|d| d.subgraph().name().as_str())
                .join(", "),
        ));
    }

    let object_id = ctx.insert_object(first.name());

    for key in definitions
        .iter()
        .flat_map(|def| def.entity_keys())
        .filter(|key| key.is_resolvable())
    {
        ctx.insert_resolvable_key(object_id, key.id);
    }
}

fn merge_field_definitions<'a>(fields: &[FieldWalker<'a>], ctx: &mut Context<'a>) {
    let Some(first) = fields.first() else { return };

    if first.parent_definition().kind() == DefinitionKind::Object {
        object::compose_object_fields(*first, fields, ctx);
    }
}

fn merge_union_definitions(
    ctx: &mut Context<'_>,
    first_union: &DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
) {
    let union_name = first_union.name();
    ctx.insert_union(union_name);

    for member in definitions
        .iter()
        .flat_map(|def| ctx.subgraphs.iter_union_members(def.id))
    {
        let member = first_union.walk(member);
        ctx.insert_union_member(union_name.id, member.name().id);
    }
}

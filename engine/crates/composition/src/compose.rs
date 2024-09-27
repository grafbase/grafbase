mod context;
mod directives;
mod entity_interface;
mod enums;
mod fields;
mod input_object;
mod interface;
mod object;
mod roots;
mod scalar;

pub(crate) use self::context::Context as ComposeContext;

use self::{context::Context, directives::collect_composed_directives, input_object::*};
use crate::{
    composition_ir as ir,
    subgraphs::{self, DefinitionKind, DefinitionWalker, FieldWalker, StringId},
};
use graphql_federated_graph as federated;
use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub(crate) fn compose_subgraphs(ctx: &mut Context<'_>) {
    ctx.subgraphs.iter_definition_groups(|definitions| {
        let Some(first) = definitions.first() else {
            return;
        };

        if first.directives().interface_object()
            || (first.kind() == DefinitionKind::Interface && first.entity_keys().next().is_some())
        {
            return entity_interface::merge_entity_interface_definitions(ctx, *first, definitions);
        }

        match first.kind() {
            DefinitionKind::Object => merge_object_definitions(ctx, first, definitions),
            DefinitionKind::Union => merge_union_definitions(ctx, first, definitions),
            DefinitionKind::InputObject => merge_input_object_definitions(ctx, first, definitions),
            DefinitionKind::Interface => interface::merge_interface_definitions(ctx, first, definitions),
            DefinitionKind::Scalar => scalar::merge_scalar_definitions(*first, definitions, ctx),
            DefinitionKind::Enum => enums::merge_enum_definitions(first, definitions, ctx),
        }
    });

    roots::merge_root_fields(ctx);
}

fn merge_object_definitions<'a>(
    ctx: &mut Context<'a>,
    first: &DefinitionWalker<'a>,
    definitions: &[DefinitionWalker<'a>],
) {
    let is_shareable = definitions.iter().any(|definition| definition.directives().shareable());

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
        return;
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

    let description = definitions.iter().find_map(|def| def.description());
    let composed_directives = collect_composed_directives(definitions.iter().map(|def| def.directives()), ctx);

    let object_name = ctx.insert_string(first.name().id);
    let object_id = ctx.insert_object(object_name, description, composed_directives);

    for key in definitions.iter().flat_map(|def| def.entity_keys()) {
        ctx.insert_key(object_id, key);
    }

    for authorized in definitions
        .iter()
        .map(|def| def.directives())
        .filter(|directives| directives.authorized().is_some())
    {
        ctx.insert_object_authorized(object_id, authorized.id);
    }

    if is_shareable {
        object::validate_shareable_object_fields_match(definitions, ctx);
    }

    fields::for_each_field_group(definitions, |fields| {
        let Some(first) = fields.first() else { return };

        object::compose_object_fields(object_id, is_shareable, *first, fields, ctx);
    });
}

fn merge_union_definitions(
    ctx: &mut Context<'_>,
    first_union: &DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
) {
    let union_name = ctx.insert_string(first_union.name().id);

    let description = definitions.iter().find_map(|def| def.description());
    let directives = collect_composed_directives(definitions.iter().map(|def| def.directives()), ctx);

    ctx.insert_union(union_name, directives, description);

    for member in definitions
        .iter()
        .flat_map(|def| ctx.subgraphs.iter_union_members(def.id))
    {
        let member = first_union.walk(member);
        let member_name = ctx.insert_string(member.name().id);
        ctx.insert_union_member(member.subgraph_id(), union_name, member_name);
    }
}

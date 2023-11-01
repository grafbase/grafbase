mod enums;
mod input_object;
mod interface;
mod object;

use self::input_object::*;
use crate::{
    subgraphs::{DefinitionKind, DefinitionWalker, FieldWalker},
    Context,
};
use itertools::Itertools;

pub(crate) fn build_supergraph(ctx: &mut Context<'_>) {
    ctx.subgraphs.iter_definition_groups(|definitions| {
        let Some(first) = definitions.get(0) else {
            return;
        };

        match first.kind() {
            DefinitionKind::Object => merge_object_definitions(ctx, first, definitions),
            DefinitionKind::Union => merge_union_definitions(ctx, first, definitions),
            DefinitionKind::InputObject => merge_input_object_definitions(ctx, first, definitions),
            DefinitionKind::Interface => {
                interface::merge_interface_definitions(ctx, first, definitions)
            }
            DefinitionKind::Scalar => {
                ctx.supergraph
                    .insert_definition(first.name(), DefinitionKind::Scalar);
            }
            DefinitionKind::Enum => enums::merge_enum_definitions(first, definitions, ctx),
        }
    });

    ctx.subgraphs
        .iter_field_groups(|fields| merge_field_definitions(ctx, fields));
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
        let name = first.name_str();
        let first_subgraph = first.subgraph().name_str();
        let second_subgraph = incompatible.subgraph().name_str();
        ctx.diagnostics.push_fatal(format!(
            "Cannot merge {first_kind:?} with {second_kind:?} (`{name}` in `{first_subgraph}` and `{second_subgraph}`)",
        ));
    }

    let first_is_entity = first.is_entity();
    if definitions
        .iter()
        .any(|object| object.is_entity() != first_is_entity)
    {
        let name = first.name_str();
        let (entity_subgraphs, non_entity_subgraphs) = definitions
            .iter()
            .partition::<Vec<DefinitionWalker<'_>>, _>(|definition| definition.is_entity());

        ctx.diagnostics.push_fatal(format!(
            "The `{name}` object is an entity in subgraphs {} but not in subgraphs {}.",
            entity_subgraphs
                .into_iter()
                .map(|d| d.subgraph().name_str())
                .join(", "),
            non_entity_subgraphs
                .into_iter()
                .map(|d| d.subgraph().name_str())
                .join(", "),
        ));
    }

    ctx.supergraph
        .insert_definition(first.name(), DefinitionKind::Object);
}

fn merge_field_definitions(ctx: &mut Context<'_>, fields: &[FieldWalker<'_>]) {
    let Some(first) = fields.get(0) else { return };

    if first.parent_definition().kind() != DefinitionKind::Object {
        return;
    }

    if fields.len() > 1 && fields.iter().any(|f| !(f.is_shareable() || f.is_key())) {
        let next = &fields[1];

        ctx.diagnostics.push_fatal(format!(
            "The field `{}` on `{}` is defined in two subgraphs (`{}` and `{}`).",
            first.name_str(),
            first.parent_definition().name_str(),
            first.parent_definition().subgraph().name_str(),
            next.parent_definition().subgraph().name_str(),
        ));
    }

    let first_is_key = first.is_key();
    if fields.iter().any(|field| field.is_key() != first_is_key) {
        let name = format!(
            "{}.{}",
            first.parent_definition().name_str(),
            first.name_str()
        );
        let (key_subgraphs, non_key_subgraphs) = fields
            .iter()
            .partition::<Vec<FieldWalker<'_>>, _>(|field| field.is_key());

        ctx.diagnostics.push_fatal(format!(
            "The field `{name}` is part of `@key` in {} but not in {}",
            key_subgraphs
                .into_iter()
                .map(|f| f.parent_definition().subgraph().name_str())
                .join(", "),
            non_key_subgraphs
                .into_iter()
                .map(|f| f.parent_definition().subgraph().name_str())
                .join(", "),
        ));
    }

    let arguments = object::merge_field_arguments(*first, fields);

    ctx.supergraph.insert_field(
        first.parent_definition().name(),
        first.name(),
        first.r#type().type_name(),
        arguments,
    )
}

fn merge_union_definitions(
    ctx: &mut Context<'_>,
    first_union: &DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
) {
    let union_name = first_union.name();
    ctx.supergraph
        .insert_definition(union_name, DefinitionKind::Union);

    for member in definitions
        .iter()
        .flat_map(|def| ctx.subgraphs.iter_union_members(def.id))
    {
        let member = first_union.walk(member);
        ctx.supergraph
            .insert_union_member(union_name, member.name());
    }
}

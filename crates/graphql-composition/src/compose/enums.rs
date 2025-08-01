use std::collections::HashSet;

use super::*;
use crate::subgraphs::{StringId, Subgraphs};

pub(super) fn merge_enum_definitions<'a>(
    first: &DefinitionWalker<'a>,
    definitions: &[DefinitionWalker<'a>],
    ctx: &mut Context<'a>,
) {
    let enum_name = first.name().id;
    let mut directives = collect_composed_directives(definitions.iter().map(|def| def.directives()), ctx);
    directives.extend(create_join_type_from_definitions(definitions));
    let description = definitions.iter().find_map(|def| def.description()).map(|d| d.as_str());

    match (
        enum_is_used_in_input(enum_name, ctx.subgraphs),
        enum_is_used_in_return_position(enum_name, ctx.subgraphs),
    ) {
        (true, false) => {
            let enum_id = ctx.insert_enum(first.name().as_str(), description, directives);
            merge_intersection(first, definitions, enum_id, ctx);
        }
        (false, true) => {
            let enum_id = ctx.insert_enum(first.name().as_str(), description, directives);
            merge_union(first, definitions, enum_id, ctx);
        }
        (true, true) => {
            let enum_id = ctx.insert_enum(first.name().as_str(), description, directives);
            merge_exactly_matching(first, definitions, enum_id, ctx);
        }
        (false, false) => {
            // The enum isn't used at all, act as if it were used in return position
            let enum_id = ctx.insert_enum(first.name().as_str(), description, directives);
            merge_union(first, definitions, enum_id, ctx);
        }
    }
}

/// Returns whether the enum is used anywhere in a field argument or an input type field.
fn enum_is_used_in_input(enum_name: StringId, subgraphs: &Subgraphs) -> bool {
    let in_field_arguments = || {
        subgraphs
            .iter_all_field_arguments()
            .any(|arg| arg.r#type().type_name().id == enum_name)
    };
    let in_input_type_fields = || {
        subgraphs
            .iter_all_fields()
            .filter(|field| field.parent_definition().kind() == DefinitionKind::InputObject)
            .any(|field| field.r#type().type_name().id == enum_name)
    };
    in_field_arguments() || in_input_type_fields()
}

/// Returns whether the enum is returned by a field anywhere.
fn enum_is_used_in_return_position(enum_name: StringId, subgraphs: &Subgraphs) -> bool {
    subgraphs
        .iter_all_fields()
        .filter(|field| {
            matches!(
                field.parent_definition().kind(),
                DefinitionKind::Object | DefinitionKind::Interface
            )
        })
        .any(|field| field.r#type().type_name().id == enum_name)
}

fn merge_intersection<'a>(
    first: &DefinitionWalker<'a>,
    definitions: &[DefinitionWalker<'a>],
    enum_id: federated::EnumDefinitionId,
    ctx: &mut Context<'a>,
) {
    let mut intersection: Vec<StringId> = first.enum_values().map(|value| value.name().id).collect();
    let mut scratch = HashSet::new();

    for definition in definitions {
        scratch.clear();
        scratch.extend(definition.enum_values().map(|val| val.name().id));
        intersection.retain(|elem| scratch.contains(elem));
    }

    if intersection.is_empty() {
        ctx.diagnostics.push_fatal(format!(
            "Values for enum {} are empty (intersection)",
            first.name().as_str(),
        ));
    }

    for value in intersection {
        let sites = definitions
            .iter()
            .filter_map(|enm| enm.enum_value_by_name(value))
            .map(|value| value.directives());
        let composed_directives = collect_composed_directives(sites, ctx);
        ctx.insert_enum_value(first.walk(value).as_str(), None, composed_directives, enum_id);
    }
}

fn merge_union<'a>(
    first: &DefinitionWalker<'a>,
    definitions: &[DefinitionWalker<'a>],
    enum_id: federated::EnumDefinitionId,
    ctx: &mut Context<'a>,
) {
    let mut all_values: Vec<(StringId, _)> = definitions
        .iter()
        .flat_map(|def| def.enum_values().map(|value| (value.name().id, value.directives().id)))
        .collect();

    all_values.sort();

    let mut start = 0;

    while start < all_values.len() {
        let name = all_values[start].0;
        let end = all_values[start..].partition_point(|(n, _)| *n == name) + start;
        let sites = all_values[start..end]
            .iter()
            .map(|(_, directives)| first.walk(*directives));
        let composed_directives = collect_composed_directives(sites, ctx);
        ctx.insert_enum_value(first.walk(name).as_str(), None, composed_directives, enum_id);
        start = end;
    }
}

fn merge_exactly_matching<'a>(
    first: &DefinitionWalker<'a>,
    definitions: &[DefinitionWalker<'a>],
    enum_id: federated::EnumDefinitionId,
    ctx: &mut Context<'a>,
) {
    let expected: Vec<_> = first.enum_values().map(|v| v.name().id).collect();

    for definition in definitions {
        if !expected
            .iter()
            .copied()
            .eq(definition.enum_values().map(|v| v.name().id))
        {
            ctx.diagnostics.push_fatal(format!(
                "The values of enum \"{}\" should match exactly in all subgraphs because the enum is used both in input and output positions, but they do not match in subgraphs \"{}\" and \"{}\".",
                first.name().as_str(),
                first.subgraph().name().as_str(),
                definition.subgraph().name().as_str(),
            ));
            return;
        }
    }

    for value in expected {
        let sites = definitions
            .iter()
            .filter_map(|enm| enm.enum_value_by_name(value))
            .map(|value| value.directives());
        let composed_directives = collect_composed_directives(sites, ctx);
        ctx.insert_enum_value(first.walk(value).as_str(), None, composed_directives, enum_id);
    }
}

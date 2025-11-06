use std::collections::HashSet;

use super::*;
use crate::subgraphs::{StringId, Subgraphs};

pub(super) fn merge_enum_definitions<'a>(
    first: &DefinitionView<'_>,
    definitions: &[DefinitionView<'_>],
    ctx: &mut Context<'a>,
) {
    let enum_name = first.name;
    let enum_name_str = ctx.subgraphs[enum_name].as_ref();
    let mut directives = collect_composed_directives(definitions.iter().map(|def| def.directives), ctx);
    directives.extend(create_join_type_from_definitions(definitions));
    let description = definitions
        .iter()
        .find_map(|def| def.description)
        .map(|d| ctx.subgraphs[d].as_ref());

    match (
        enum_is_used_in_input(enum_name, ctx.subgraphs),
        enum_is_used_in_return_position(enum_name, ctx.subgraphs),
    ) {
        (true, false) => {
            let enum_id = ctx.insert_enum(enum_name_str, description, directives);
            merge_intersection(first, definitions, enum_id, ctx);
        }
        (false, true) => {
            let enum_id = ctx.insert_enum(enum_name_str, description, directives);
            merge_union(definitions, enum_id, ctx);
        }
        (true, true) => {
            let enum_id = ctx.insert_enum(enum_name_str, description, directives);
            merge_exactly_matching(first, definitions, enum_id, ctx);
        }
        (false, false) => {
            // The enum isn't used at all. Act as if it were used in return position
            let enum_id = ctx.insert_enum(enum_name_str, description, directives);
            merge_union(definitions, enum_id, ctx);
        }
    }
}

/// Returns whether the enum is used anywhere in a field argument or an input type field.
fn enum_is_used_in_input(enum_name: StringId, subgraphs: &Subgraphs) -> bool {
    let in_field_arguments = || {
        subgraphs
            .iter_output_field_arguments()
            .any(|arg| arg.r#type.definition_name_id == enum_name)
    };
    let in_input_type_fields = || {
        subgraphs
            .iter_fields()
            .filter(|field| {
                let parent_definition = subgraphs.at(field.parent_definition_id);
                parent_definition.kind == DefinitionKind::InputObject
            })
            .any(|field| field.r#type.definition_name_id == enum_name)
    };
    in_field_arguments() || in_input_type_fields()
}

/// Returns whether the enum is returned by a field anywhere.
fn enum_is_used_in_return_position(enum_name: StringId, subgraphs: &Subgraphs) -> bool {
    subgraphs
        .iter_fields()
        .filter(|field| {
            let parent_definition = subgraphs.at(field.parent_definition_id);
            matches!(
                parent_definition.kind,
                DefinitionKind::Object | DefinitionKind::Interface
            )
        })
        .any(|field| field.r#type.definition_name_id == enum_name)
}

fn merge_intersection<'a>(
    first: &DefinitionView<'_>,
    definitions: &[DefinitionView<'_>],
    enum_id: federated::EnumDefinitionId,
    ctx: &mut Context<'a>,
) {
    let mut intersection: Vec<StringId> = first.id.enum_values(ctx.subgraphs).map(|value| value.name).collect();
    let mut scratch = HashSet::new();

    for definition in definitions {
        scratch.clear();
        scratch.extend(definition.id.enum_values(ctx.subgraphs).map(|val| val.name));
        intersection.retain(|elem| scratch.contains(elem));
    }

    if intersection.is_empty() {
        ctx.diagnostics.push_fatal(format!(
            "Values for enum {} are empty (intersection)",
            ctx.subgraphs[first.name],
        ));
    }

    for value in intersection {
        let value_definitions = definitions
            .iter()
            .filter_map(|enm| enm.id.enum_value_by_name(ctx.subgraphs, value));

        let sites = value_definitions.clone().map(|value| value.directives);
        let mut composed_directives = collect_composed_directives(sites, ctx);
        let mut description = None;

        for value_definition in value_definitions {
            description = description.or(value_definition.description);
            let parent_definition = ctx.subgraphs.at(value_definition.parent_enum_id);
            composed_directives.push(ir::Directive::JoinEnumValue(ir::JoinEnumValueDirective {
                graph: parent_definition.subgraph_id.idx().into(),
            }));
        }

        ctx.insert_enum_value(&ctx.subgraphs[value], description, composed_directives, enum_id);
    }
}

fn merge_union<'a>(definitions: &[DefinitionView<'_>], enum_id: federated::EnumDefinitionId, ctx: &mut Context<'a>) {
    let mut all_values: Vec<_> = definitions
        .iter()
        .flat_map(|def| def.id.enum_values(ctx.subgraphs))
        .collect();

    all_values.sort_by_key(|value| value.name);

    for values in all_values.chunk_by(|a, b| a.name == b.name) {
        let name = values[0].name;

        let sites = values.iter().map(|v| v.directives);
        let mut composed_directives = collect_composed_directives(sites, ctx);
        let mut description = None;

        for value in values {
            description = description.or(value.description);
            let parent_definition = ctx.subgraphs.at(value.parent_enum_id);
            composed_directives.push(ir::Directive::JoinEnumValue(ir::JoinEnumValueDirective {
                graph: parent_definition.subgraph_id.idx().into(),
            }));
        }

        ctx.insert_enum_value(&ctx.subgraphs[name], description, composed_directives, enum_id);
    }
}

fn merge_exactly_matching<'a>(
    first: &DefinitionView<'_>,
    definitions: &[DefinitionView<'_>],
    enum_id: federated::EnumDefinitionId,
    ctx: &mut Context<'a>,
) {
    let expected: Vec<_> = first.id.enum_values(ctx.subgraphs).map(|v| v.name).collect();

    for definition in definitions {
        if !expected
            .iter()
            .copied()
            .eq(definition.id.enum_values(ctx.subgraphs).map(|v| v.name))
        {
            ctx.diagnostics.push_fatal(format!(
                "The values of enum \"{}\" should match exactly in all subgraphs because the enum is used both in input and output positions, but they do not match in subgraphs \"{}\" and \"{}\".",
                ctx.subgraphs[first.name],
                ctx.subgraphs[ctx.subgraphs.at(first.subgraph_id).name],
                ctx.subgraphs[ctx.subgraphs.at(definition.subgraph_id).name],
            ));
            return;
        }
    }

    for value in expected {
        let enum_value_definitions = definitions
            .iter()
            .filter_map(|enm| enm.id.enum_value_by_name(ctx.subgraphs, value));

        let sites = enum_value_definitions.clone().map(|value| value.directives);
        let mut composed_directives = collect_composed_directives(sites, ctx);

        let mut description = None;

        for value_definition in enum_value_definitions {
            description = description.or(value_definition.description);
            let parent_definition = ctx.subgraphs.at(value_definition.parent_enum_id);
            composed_directives.push(ir::Directive::JoinEnumValue(ir::JoinEnumValueDirective {
                graph: parent_definition.subgraph_id.idx().into(),
            }))
        }

        ctx.insert_enum_value(&ctx.subgraphs[value], description, composed_directives, enum_id);
    }
}

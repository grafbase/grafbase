use std::collections::HashSet;

use super::*;
use crate::subgraphs::{StringId, Subgraphs};

pub(super) fn merge_enum_definitions(
    first: &DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
    ctx: &mut Context<'_>,
) {
    let enum_name = first.name().id;
    let is_inaccessible = definitions.iter().any(|definition| definition.is_inaccessible());

    match (
        enum_is_used_in_input(enum_name, ctx.subgraphs),
        enum_is_used_in_return_position(enum_name, ctx.subgraphs),
    ) {
        (true, false) => merge_intersection(first, definitions, is_inaccessible, ctx),
        (false, true) => merge_union(first, definitions, is_inaccessible, ctx),
        (true, true) => merge_exactly_matching(first, definitions, is_inaccessible, ctx),
        (false, false) => {
            // The enum isn't used at all, omit it from the federated graph
        }
    }
}

/// Returns whether the enum is used anywhere in a field argument or an input type field.
fn enum_is_used_in_input(enum_name: StringId, subgraphs: &Subgraphs) -> bool {
    let in_field_arguments = || {
        subgraphs
            .iter_fields()
            .flat_map(|field| field.arguments())
            .any(|arg| arg.argument_type().type_name().id == enum_name)
    };
    let in_input_type_fields = || {
        subgraphs
            .iter_fields()
            .filter(|field| field.parent_definition().kind() == DefinitionKind::InputObject)
            .any(|field| field.r#type().type_name().id == enum_name)
    };
    in_field_arguments() || in_input_type_fields()
}

/// Returns whether the enum is returned by a field anywhere.
fn enum_is_used_in_return_position(enum_name: StringId, subgraphs: &Subgraphs) -> bool {
    subgraphs
        .iter_fields()
        .filter(|field| {
            matches!(
                field.parent_definition().kind(),
                DefinitionKind::Object | DefinitionKind::Interface
            )
        })
        .any(|field| field.r#type().type_name().id == enum_name)
}

fn merge_intersection(
    first: &DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
    is_inaccessible: bool,
    ctx: &mut Context<'_>,
) {
    let mut intersection: Vec<StringId> = first.enum_values().collect();
    let mut scratch = HashSet::new();

    for definition in definitions {
        scratch.clear();
        scratch.extend(definition.enum_values());
        intersection.retain(|elem| scratch.contains(elem));
    }

    if intersection.is_empty() {
        ctx.diagnostics.push_fatal(format!(
            "Values for enum {} are empty (intersection)",
            first.name().as_str(),
        ));
    }

    let enum_id = ctx.insert_enum(first.name(), is_inaccessible);

    for value in intersection {
        let deprecation = ctx.subgraphs.get_enum_value_deprecation((first.name().id, value));
        ctx.insert_enum_value(enum_id, first.walk(value), deprecation);
    }
}

fn merge_union(
    first: &DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
    is_inaccessible: bool,
    ctx: &mut Context<'_>,
) {
    let enum_id = ctx.insert_enum(first.name(), is_inaccessible);

    for value in definitions.iter().flat_map(|def| def.enum_values()) {
        let deprecation = ctx.subgraphs.get_enum_value_deprecation((first.name().id, value));
        ctx.insert_enum_value(enum_id, first.walk(value), deprecation);
    }
}

fn merge_exactly_matching(
    first: &DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
    is_inaccessible: bool,
    ctx: &mut Context<'_>,
) {
    let expected: Vec<_> = first.enum_values().collect();

    for definition in definitions {
        if !is_slice_match(&expected, definition.enum_values()) {
            ctx.diagnostics.push_fatal(format!(
                "The enum {} should match exactly in all subgraphs, but it does not",
                first.name().as_str()
            ));
            return;
        }
    }

    let enum_id = ctx.insert_enum(first.name(), is_inaccessible);

    for value in expected {
        let deprecation = ctx.subgraphs.get_enum_value_deprecation((first.name().id, value));
        ctx.insert_enum_value(enum_id, first.walk(value), deprecation);
    }
}

fn is_slice_match<T: PartialEq>(slice: &[T], iterator: impl Iterator<Item = T>) -> bool {
    let mut idx = 0;

    for item in iterator {
        if slice[idx] != item {
            return false;
        }

        idx += 1;
    }

    idx == slice.len()
}

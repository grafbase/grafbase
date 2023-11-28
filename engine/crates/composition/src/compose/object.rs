use super::*;
use crate::{
    composition_ir as ir,
    subgraphs::{FieldTypeId, StringId},
};
use std::collections::{BTreeSet, HashSet};

/// The arguments of a federated graph's fields are the interseciton of the subgraph's arguments for
/// that field.
pub(super) fn merge_field_arguments<'a>(
    first: FieldWalker<'a>,
    fields: &[FieldWalker<'a>],
) -> Vec<(StringId, FieldTypeId)> {
    let mut intersection: Vec<_> = first.arguments().map(|arg| arg.argument_name().id).collect();
    let mut buf = HashSet::new();

    for field in &fields[1..] {
        buf.clear();
        buf.extend(field.arguments().map(|arg| arg.argument_name().id));
        intersection.retain(|value| buf.contains(value));
    }

    first
        .arguments()
        .filter(|arg| intersection.contains(&arg.argument_name().id))
        .map(|arg| (arg.argument_name().id, arg.argument_type().id))
        .collect()
}

pub(super) fn compose_object_fields<'a>(first: FieldWalker<'a>, fields: &[FieldWalker<'a>], ctx: &mut Context<'a>) {
    if fields
        .iter()
        .filter(|f| !(f.is_shareable() || f.is_external() || f.is_part_of_key() || f.overrides().is_some()))
        .count()
        > 1
    {
        let next = &fields[1];

        ctx.diagnostics.push_fatal(format!(
            "The field `{}` on `{}` is defined in two subgraphs (`{}` and `{}`).",
            first.name().as_str(),
            first.parent_definition().name().as_str(),
            first.parent_definition().subgraph().name().as_str(),
            next.parent_definition().subgraph().name().as_str(),
        ));
    }

    let first_is_part_of_key = first.is_part_of_key();
    if fields
        .iter()
        .any(|field| field.is_part_of_key() != first_is_part_of_key)
    {
        let name = format!(
            "{}.{}",
            first.parent_definition().name().as_str(),
            first.name().as_str()
        );
        let (key_subgraphs, non_key_subgraphs) = fields
            .iter()
            .partition::<Vec<FieldWalker<'_>>, _>(|field| field.is_part_of_key());

        ctx.diagnostics.push_fatal(format!(
            "The field `{name}` is part of `@key` in {} but not in {}",
            key_subgraphs
                .into_iter()
                .map(|f| f.parent_definition().subgraph().name().as_str())
                .join(", "),
            non_key_subgraphs
                .into_iter()
                .map(|f| f.parent_definition().subgraph().name().as_str())
                .join(", "),
        ));
    }

    let arguments = object::merge_field_arguments(first, fields);

    let resolvable_in = fields
        .first()
        .filter(|_| fields.len() == 1)
        .map(|field| federated::SubgraphId(field.parent_definition().subgraph().id.idx()));

    let provides = fields.iter().filter(|f| f.provides().is_some()).map(|f| f.id).collect();

    let requires = fields.iter().filter(|f| f.requires().is_some()).map(|f| f.id).collect();

    let mut composed_directives = Vec::new();

    if let Some(reason) = fields.iter().find_map(|f| f.deprecated().map(|d| d.map(|d| d.id))) {
        let arguments = match reason {
            Some(reason) => vec![(
                ctx.insert_static_str("reason"),
                federated::Value::String(ctx.insert_string(reason)),
            )],
            None => Vec::new(),
        };
        let directive_name = ctx.insert_static_str("deprecated");
        composed_directives.push(federated::Directive {
            name: directive_name,
            arguments,
        });
    }

    for tag in fields
        .iter()
        .flat_map(|f| f.tags().map(|t| t.id))
        .collect::<BTreeSet<_>>()
    {
        let directive_name = ctx.insert_static_str("tag");
        let argument_name = ctx.insert_static_str("name");
        let tag_argument = federated::Value::String(ctx.insert_string(tag));
        composed_directives.push(federated::Directive {
            name: directive_name,
            arguments: vec![(argument_name, tag_argument)],
        });
    }

    let overrides = collect_overrides(fields, ctx);

    ctx.insert_field(ir::FieldIr {
        parent_name: first.parent_definition().name().id,
        field_name: first.name().id,
        field_type: first.r#type().id,
        arguments,
        resolvable_in,
        provides,
        requires,
        composed_directives,
        overrides,
    });
}

fn collect_overrides(fields: &[FieldWalker<'_>], ctx: &mut Context<'_>) -> Vec<federated::Override> {
    let mut overrides = Vec::new();

    for (field, from) in fields.iter().filter_map(|f| Some(f).zip(f.overrides())) {
        let field_subgraph = field.parent_definition().subgraph();

        if from.id == field_subgraph.name().id {
            ctx.diagnostics.push_fatal(format!(
                r#"Source and destination subgraphs "{}" are the same for overridden field "{}.{}""#,
                from.as_str(),
                field.parent_definition().name().as_str(),
                field.name().as_str()
            ));
            continue;
        }

        if let Some(override_source) = fields
            .iter()
            .find(|f| f.parent_definition().subgraph().name().id == from.id)
        {
            if override_source.overrides().is_some() {
                ctx.diagnostics
                    .push_fatal(format!(r#"Field "{}.{}" on subgraph "{}" is also marked with directive @override in subgraph "{}". Only one @override directive is allowed per field."#,
                        override_source.parent_definition().name().as_str(),
                        override_source.name().as_str(),
                        override_source.parent_definition().subgraph().name().as_str(),
                        field.parent_definition().subgraph().name().as_str()));
            }
        }

        overrides.push(federated::Override {
            graph: federated::SubgraphId(field_subgraph.id.idx()),
            from: ctx.insert_string(from.id),
        });
    }

    overrides
}

use std::collections::HashSet;

use super::*;
use crate::subgraphs::{FieldTypeId, StringId};

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
        .filter(|f| !(f.is_shareable() || f.is_external() || f.is_key()))
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

    let first_is_key = first.is_key();
    if fields.iter().any(|field| field.is_key() != first_is_key) {
        let name = format!(
            "{}.{}",
            first.parent_definition().name().as_str(),
            first.name().as_str()
        );
        let (key_subgraphs, non_key_subgraphs) = fields
            .iter()
            .partition::<Vec<FieldWalker<'_>>, _>(|field| field.is_key());

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
        .map(|field| graphql_federated_graph::SubgraphId(field.parent_definition().subgraph().id.idx()));
    let provides = fields.iter().filter(|f| f.provides().is_some()).map(|f| f.id).collect();

    ctx.insert_field(
        first.parent_definition().name().id,
        first.name().id,
        first.r#type().id,
        arguments,
        resolvable_in,
        provides,
    );
}

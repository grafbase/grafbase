use super::*;
use crate::subgraphs::{FieldTypeId, StringId};
use std::collections::HashSet;

/// The arguments of a federated graph's fields are the interseciton of the subgraph's arguments for
/// that field.
pub(super) fn merge_field_arguments<'a>(
    first: FieldWalker<'a>,
    fields: &[FieldWalker<'a>],
) -> Vec<(StringId, FieldTypeId)> {
    let mut intersection: Vec<_> = first
        .arguments()
        .map(|arg| arg.argument_name().id)
        .collect();
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

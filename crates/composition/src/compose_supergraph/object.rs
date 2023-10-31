use super::*;
use crate::strings::StringId;
use std::collections::HashSet;

/// The arguments of a supergraph's fields are the interseciton of the subgraph's arguments for
/// that field.
pub(super) fn merge_field_arguments(
    first: FieldWalker<'_>,
    fields: &[FieldWalker<'_>],
) -> Vec<(StringId, StringId)> {
    let mut intersection: HashSet<_> = first.arguments().map(|arg| arg.argument_name()).collect();
    let mut buf = HashSet::new();

    for field in &fields[1..] {
        buf.clear();
        buf.extend(field.arguments().map(|arg| arg.argument_name()));
        intersection.retain(|value| buf.contains(value));
    }

    first
        .arguments()
        .filter(|arg| intersection.contains(&arg.argument_name()))
        .map(|arg| (arg.argument_name(), arg.argument_type().type_name()))
        .collect()
}

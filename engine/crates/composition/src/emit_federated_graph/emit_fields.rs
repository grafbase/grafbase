use crate::composition_ir::FieldIr;
use graphql_federated_graph as federated;

/// Takes each group of consecutive fields with the same parent definition and calls `f` with each.
pub(super) fn for_each_field_group<F>(fields: &[FieldIr], mut f: F)
where
    F: FnMut(federated::Definition, &mut Vec<FieldIr>),
{
    let mut start = 0;
    let mut buf = Vec::new();

    while start < fields.len() {
        let definition = fields[start].parent_definition;
        let end = start + fields[start..].partition_point(|field| field.parent_definition == definition);
        buf.extend_from_slice(&fields[start..end]);

        f(definition, &mut buf);

        buf.clear();
        start = end;
    }
}

use crate::{composition_ir::FieldIr, federated_graph as federated};

/// Takes each group of consecutive fields with the same parent definition and calls `f` with each.
pub(super) fn for_each_field_group<F>(fields: &[FieldIr], mut f: F)
where
    F: FnMut(federated::StringId, &[FieldIr]),
{
    for chunk in fields.chunk_by(|a, b| a.parent_definition_name == b.parent_definition_name) {
        let parent_definition_name = chunk[0].parent_definition_name;
        f(parent_definition_name, chunk)
    }
}

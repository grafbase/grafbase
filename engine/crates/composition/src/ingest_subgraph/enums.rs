use super::*;
use crate::subgraphs::*;

pub(super) fn ingest_enum(definition_id: DefinitionId, enum_type: &ast::EnumType, subgraphs: &mut Subgraphs) {
    for value in &enum_type.values {
        let value_name = subgraphs.strings.intern(value.node.value.node.as_str());
        subgraphs.push_enum_value(definition_id, value_name);

        if let Some(deprecated) = super::find_deprecated_directive(&value.node.directives, subgraphs) {
            subgraphs.deprecate_enum_value((subgraphs.walk(definition_id).name().id, value_name), deprecated);
        }
    }
}

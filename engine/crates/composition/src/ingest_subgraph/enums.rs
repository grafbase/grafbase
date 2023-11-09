use super::*;
use crate::subgraphs::*;

pub(super) fn ingest_enum(definition_id: DefinitionId, enum_type: &ast::EnumType, subgraphs: &mut Subgraphs) {
    for value in &enum_type.values {
        let value = subgraphs.strings.intern(value.node.value.node.as_str());
        subgraphs.push_enum_value(definition_id, value);
    }
}

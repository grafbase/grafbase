use super::field_types_map::FieldTypesMap;
use crate::{composition_ir as ir, subgraphs};
use graphql_federated_graph as federated;
use std::collections::HashMap;

pub(super) struct Context<'a> {
    pub(super) out: &'a mut federated::FederatedGraph,
    pub(super) subgraphs: &'a subgraphs::Subgraphs,
    pub(super) field_types_map: FieldTypesMap,
    pub(super) selection_map: HashMap<(federated::Definition, federated::StringId), federated::FieldId>,
    pub(super) definitions: HashMap<federated::StringId, federated::Definition>,

    strings_ir: ir::StringsIr,
}

impl<'a> Context<'a> {
    pub(crate) fn new(
        ir: &mut ir::CompositionIr,
        subgraphs: &'a subgraphs::Subgraphs,
        out: &'a mut federated::FederatedGraph,
    ) -> Self {
        Context {
            out,
            subgraphs,
            definitions: std::mem::take(&mut ir.definitions_by_name),
            strings_ir: std::mem::take(&mut ir.strings),
            selection_map: HashMap::with_capacity(ir.fields.len()),
            field_types_map: FieldTypesMap::default(),
        }
    }

    /// Subgraphs string -> federated graph string.
    pub(crate) fn insert_string(&mut self, string: subgraphs::StringWalker<'_>) -> federated::StringId {
        self.strings_ir.insert(string.as_str())
    }

    pub(crate) fn lookup_str(&self, string: &str) -> Option<federated::StringId> {
        self.strings_ir.lookup(string)
    }

    pub(crate) fn lookup_string_id(&self, string: federated::StringId) -> &str {
        &self.strings_ir[string]
    }

    pub(crate) fn insert_value(&mut self, value: &subgraphs::Value) -> federated::Value {
        self.insert_value_with_type(value, None)
    }

    pub(crate) fn insert_value_with_type(
        &mut self,
        value: &subgraphs::Value,
        enum_type: Option<federated::TypeDefinitionId>,
    ) -> federated::Value {
        match value {
            subgraphs::Value::Null => federated::Value::Null,
            subgraphs::Value::String(value) => {
                federated::Value::String(self.insert_string(self.subgraphs.walk(*value)))
            }
            subgraphs::Value::Int(value) => federated::Value::Int(*value),
            subgraphs::Value::Float(value) => federated::Value::Float(*value),
            subgraphs::Value::Boolean(value) => federated::Value::Boolean(*value),
            subgraphs::Value::Enum(value) => {
                let value_name = self.insert_string(self.subgraphs.walk(*value));
                let enum_value = enum_type.and_then(|enum_id| self.out.find_enum_value_by_name_id(enum_id, value_name));

                if let Some(enum_value) = enum_value {
                    federated::Value::EnumValue(enum_value.id())
                } else {
                    federated::Value::UnboundEnumValue(value_name)
                }
            }
            subgraphs::Value::Object(value) => federated::Value::Object(
                value
                    .iter()
                    .map(|(k, v)| {
                        (
                            self.insert_string(self.subgraphs.walk(*k)),
                            self.insert_value_with_type(v, enum_type),
                        )
                    })
                    .collect(),
            ),
            subgraphs::Value::List(value) => federated::Value::List(
                value
                    .iter()
                    .map(|v| self.insert_value_with_type(v, enum_type))
                    .collect(),
            ),
        }
    }
}

impl std::ops::Index<federated::StringId> for Context<'_> {
    type Output = str;

    fn index(&self, index: federated::StringId) -> &Self::Output {
        &self.strings_ir[index]
    }
}

impl Drop for Context<'_> {
    fn drop(&mut self) {
        self.out.strings = std::mem::take(&mut self.strings_ir).into_federated_strings();
    }
}

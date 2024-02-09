use super::field_types_map::FieldTypesMap;
use crate::{composition_ir as ir, subgraphs};
use graphql_federated_graph as federated;
use std::collections::HashMap;

pub(super) struct Context<'a> {
    pub(super) out: &'a mut federated::FederatedGraphV2,
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
        out: &'a mut federated::FederatedGraphV2,
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

    pub(crate) fn push_object_field(&mut self, object_id: federated::ObjectId, field_id: federated::FieldId) {
        let key = (federated::Definition::Object(object_id), self.out[field_id].name);
        self.selection_map.insert(key, field_id);
        self.out
            .object_fields
            .push(federated::ObjectField { object_id, field_id });
    }

    pub(crate) fn push_interface_field(&mut self, interface_id: federated::InterfaceId, field_id: federated::FieldId) {
        let key = (federated::Definition::Interface(interface_id), self.out[field_id].name);
        self.selection_map.insert(key, field_id);
        self.out
            .interface_fields
            .push(federated::InterfaceField { interface_id, field_id });
    }
}

impl Drop for Context<'_> {
    fn drop(&mut self) {
        self.out.strings = std::mem::take(&mut self.strings_ir).into_federated_strings();
    }
}

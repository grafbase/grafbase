use std::collections::HashSet;

use crate::{
    composition_ir::CompositionIr,
    subgraphs::{self, DefinitionId, StringId, StringWalker, Walker},
    Diagnostics,
};
use graphql_federated_graph as federated;

/// Context for [`compose`](crate::compose::compose).
pub(crate) struct Context<'a> {
    pub(crate) subgraphs: &'a subgraphs::Subgraphs,
    pub(crate) diagnostics: &'a mut Diagnostics,
    pub(crate) inaccessible_definitions: HashSet<StringId>,

    /// This should stay private, composition IR should remain write-only during composition: the
    /// subgraphs are the source of truth.
    ir: CompositionIr,
}

impl<'a> Context<'a> {
    pub(crate) fn new(subgraphs: &'a subgraphs::Subgraphs, diagnostics: &'a mut Diagnostics) -> Self {
        let mut ir = CompositionIr::default();

        for builtin_scalar in subgraphs.iter_builtin_scalars() {
            ir.insert_scalar(builtin_scalar);
        }

        Context {
            subgraphs,
            diagnostics,
            ir,
            inaccessible_definitions: HashSet::new(),
        }
    }

    pub(crate) fn has_query_type(&self) -> bool {
        self.ir.query_type.is_some()
    }

    pub(crate) fn into_ir(self) -> CompositionIr {
        self.ir
    }

    pub(crate) fn insert_enum(&mut self, name: StringWalker<'_>) -> federated::EnumId {
        self.ir.insert_enum(name)
    }

    pub(crate) fn insert_enum_value(
        &mut self,
        enum_id: federated::EnumId,
        value: StringWalker<'_>,
        deprecation: Option<Option<StringWalker<'_>>>,
    ) {
        self.ir.insert_enum_value(enum_id, value, deprecation);
    }

    pub(crate) fn insert_field(&mut self, ir: crate::composition_ir::FieldIr) {
        self.ir.insert_field(ir);
    }

    pub(crate) fn insert_input_object(&mut self, name: StringWalker<'_>) -> federated::InputObjectId {
        self.ir.insert_input_object(name)
    }

    pub(crate) fn insert_interface(&mut self, name: StringWalker<'_>) -> federated::InterfaceId {
        self.ir.insert_interface(name)
    }

    pub(crate) fn insert_object(&mut self, name: StringWalker<'_>, is_inaccessible: bool) -> federated::ObjectId {
        self.ir.insert_object(name, is_inaccessible)
    }

    pub(crate) fn insert_scalar(&mut self, name: StringWalker<'_>) {
        self.ir.insert_scalar(name);
    }

    pub(crate) fn insert_union(&mut self, name: StringWalker<'_>) -> federated::UnionId {
        self.ir.insert_union(name)
    }

    pub(crate) fn insert_union_member(&mut self, union_name: subgraphs::StringId, member_name: subgraphs::StringId) {
        self.ir.insert_union_member(union_name, member_name);
    }

    pub(crate) fn insert_resolvable_key(&mut self, object_id: federated::ObjectId, key_id: subgraphs::KeyId) {
        self.ir.insert_resolvable_key(object_id, key_id);
    }

    pub(crate) fn insert_string(&mut self, string_id: subgraphs::StringId) -> federated::StringId {
        self.ir.insert_string(self.subgraphs.walk(string_id))
    }

    // We need a separate method for strings that appear in the federated graph but were not
    // interned in subgraphs.
    pub(crate) fn insert_static_str(&mut self, string: &'static str) -> federated::StringId {
        match self.subgraphs.strings.lookup(string) {
            Some(id) => self.ir.insert_string(self.subgraphs.walk(id)),
            None => self.ir.insert_static_str(string),
        }
    }

    /// note: only populated within `compose_graphs`
    pub(crate) fn set_inaccessible_definitions(&mut self, definitions: Vec<Walker<'_, DefinitionId>>) {
        self.inaccessible_definitions = HashSet::from_iter(definitions.iter().map(|definition| definition.name().id))
    }

    /// note: only works within `compose_graphs`
    pub(crate) fn has_inaccessible_definition(&mut self, definition_string_id: StringId) -> bool {
        self.inaccessible_definitions.contains(&definition_string_id)
    }
}

impl Context<'_> {}

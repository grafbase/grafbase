use crate::{
    composition_ir::CompositionIr,
    subgraphs::{self, StringWalker},
    Diagnostics,
};
use grafbase_federated_graph as federated;

/// Context for [`compose`](crate::compose::compose).
pub(crate) struct Context<'a> {
    pub(crate) subgraphs: &'a subgraphs::Subgraphs,
    pub(crate) diagnostics: &'a mut Diagnostics,

    /// This should stay private, composition IR should remain write-only during composition: the
    /// subgraphs are the source of truth.
    ir: CompositionIr,
}

impl<'a> Context<'a> {
    pub(crate) fn new(
        subgraphs: &'a subgraphs::Subgraphs,
        diagnostics: &'a mut Diagnostics,
    ) -> Self {
        let mut ir = CompositionIr::default();

        for builtin_scalar in subgraphs.iter_builtin_scalars() {
            ir.insert_scalar(builtin_scalar);
        }

        Context {
            subgraphs,
            diagnostics,
            ir,
        }
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
    ) {
        self.ir.insert_enum_value(enum_id, value)
    }

    pub(crate) fn insert_field(
        &mut self,
        parent_name: subgraphs::StringId,
        field_name: subgraphs::StringId,
        field_type: subgraphs::FieldTypeId,
        arguments: Vec<(subgraphs::StringId, subgraphs::FieldTypeId)>,
        resolvable_in: Vec<federated::SubgraphId>,
    ) {
        self.ir.insert_field(
            parent_name,
            field_name,
            field_type,
            arguments,
            resolvable_in,
        )
    }

    pub(crate) fn insert_input_object(
        &mut self,
        name: StringWalker<'_>,
    ) -> federated::InputObjectId {
        self.ir.insert_input_object(name)
    }

    pub(crate) fn insert_interface(&mut self, name: StringWalker<'_>) -> federated::InterfaceId {
        self.ir.insert_interface(name)
    }

    pub(crate) fn insert_object(&mut self, name: StringWalker<'_>) -> federated::ObjectId {
        self.ir.insert_object(name)
    }

    pub(crate) fn insert_scalar(&mut self, name: StringWalker<'_>) {
        self.ir.insert_scalar(name)
    }

    pub(crate) fn insert_union(&mut self, name: StringWalker<'_>) -> federated::UnionId {
        self.ir.insert_union(name)
    }

    pub(crate) fn insert_union_member(
        &mut self,
        union_name: subgraphs::StringId,
        member_name: subgraphs::StringId,
    ) {
        self.ir.insert_union_member(union_name, member_name)
    }

    pub(crate) fn insert_resolvable_key(
        &mut self,
        object_id: federated::ObjectId,
        key_id: subgraphs::KeyId,
    ) {
        self.ir.insert_resolvable_key(object_id, key_id)
    }
}

impl Context<'_> {}

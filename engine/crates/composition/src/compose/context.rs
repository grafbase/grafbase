use std::collections::{btree_map, BTreeSet};

use crate::{
    composition_ir::{self as ir, CompositionIr},
    subgraphs::{self, StringWalker, SubgraphId},
    Diagnostics, VecExt,
};
use graphql_federated_graph as federated;

/// Context for [`compose`](crate::compose::compose).
pub(crate) struct Context<'a> {
    pub(crate) subgraphs: &'a subgraphs::Subgraphs,
    pub(crate) diagnostics: &'a mut Diagnostics,
    /// This should stay private, composition IR should remain write-only during composition: the
    /// subgraphs are the source of truth.
    ir: CompositionIr,
}

impl<'a> Context<'a> {
    pub(crate) fn new(subgraphs: &'a subgraphs::Subgraphs, diagnostics: &'a mut Diagnostics) -> Self {
        subgraphs.emit_ingestion_diagnostics(diagnostics);

        let mut context = Context {
            subgraphs,
            diagnostics,
            ir: CompositionIr::default(),
        };

        for builtin_scalar in subgraphs.iter_builtin_scalars() {
            context.insert_scalar(builtin_scalar.as_str(), None, federated::NO_DIRECTIVES);
        }

        context
    }

    pub(crate) fn into_ir(self) -> CompositionIr {
        self.ir
    }

    pub(crate) fn insert_directive(&mut self, directive: ir::Directive) -> federated::DirectiveId {
        federated::DirectiveId::from(self.ir.directives.push_return_idx(directive))
    }

    pub(crate) fn insert_enum(
        &mut self,
        enum_name: &str,
        description: Option<&str>,
        directives: federated::Directives,
    ) -> federated::TypeDefinitionId {
        let name = self.ir.strings.insert(enum_name);
        let description = description.map(|description| self.ir.strings.insert(description));

        let r#enum = federated::TypeDefinitionRecord {
            name,
            directives,
            description,
            kind: federated::TypeDefinitionKind::Enum,
        };

        let id = federated::TypeDefinitionId::from(self.ir.type_definitions.push_return_idx(r#enum));
        self.ir
            .definitions_by_name
            .insert(name, federated::Definition::Enum(id));
        id
    }

    pub(crate) fn insert_enum_value(
        &mut self,
        value: &str,
        description: Option<&str>,
        composed_directives: federated::Directives,
        enum_id: federated::TypeDefinitionId,
    ) -> federated::EnumValueId {
        let value = self.ir.strings.insert(value);
        let description = description.map(|description| self.ir.strings.insert(description));

        federated::EnumValueId::from(self.ir.enum_values.push_return_idx(federated::EnumValueRecord {
            enum_id,
            value,
            composed_directives,
            description,
        }))
    }

    pub(crate) fn insert_field(&mut self, ir: ir::FieldIr) -> federated::FieldId {
        federated::FieldId::from(self.ir.fields.push_return_idx(ir))
    }

    pub(crate) fn insert_input_object(
        &mut self,
        name: federated::StringId,
        description: Option<&str>,
        composed_directives: federated::Directives,
        fields: federated::InputValueDefinitions,
    ) -> federated::InputObjectId {
        let description = description.map(|description| self.ir.strings.insert(description));
        let object = federated::InputObject {
            name,
            fields,
            composed_directives,
            description,
        };
        let id = federated::InputObjectId::from(self.ir.input_objects.push_return_idx(object));
        self.ir
            .definitions_by_name
            .insert(name, federated::Definition::InputObject(id));
        id
    }

    pub(crate) fn insert_input_value_definition(
        &mut self,
        definition: ir::InputValueDefinitionIr,
    ) -> federated::InputValueDefinitionId {
        federated::InputValueDefinitionId::from(self.ir.input_value_definitions.push_return_idx(definition))
    }

    pub(crate) fn insert_interface(
        &mut self,
        name: federated::StringId,
        description: Option<&str>,
        composed_directives: federated::Directives,
    ) -> federated::InterfaceId {
        let description = description.map(|description| self.ir.strings.insert(description));

        let type_definition = federated::TypeDefinitionRecord {
            name,
            description,
            directives: composed_directives,
            kind: federated::TypeDefinitionKind::Interface,
        };
        let type_definition_id = self.ir.type_definitions.push_return_idx(type_definition).into();

        let interface = federated::Interface {
            type_definition_id,
            implements_interfaces: Vec::new(),
            keys: Vec::new(),
            fields: federated::NO_FIELDS,
            join_implements: Vec::new(),
        };
        let id = federated::InterfaceId::from(self.ir.interfaces.push_return_idx(interface));
        self.ir
            .definitions_by_name
            .insert(name, federated::Definition::Interface(id));
        id
    }

    pub(crate) fn insert_interface_resolvable_key(
        &mut self,
        id: federated::InterfaceId,
        key: subgraphs::KeyWalker<'_>,
        is_interface_object: bool,
    ) {
        self.ir.keys.push(ir::KeyIr {
            parent: federated::Definition::Interface(id),
            key_id: key.id,
            is_interface_object,
            resolvable: key.is_resolvable(),
        });
    }

    pub(crate) fn insert_object(
        &mut self,
        name: federated::StringId,
        description: Option<StringWalker<'_>>,
        composed_directives: federated::Directives,
    ) -> federated::ObjectId {
        let description = description.map(|description| self.ir.strings.insert(description.as_str()));
        let type_definition = federated::TypeDefinitionRecord {
            name,
            description,
            directives: composed_directives,
            kind: federated::TypeDefinitionKind::Object,
        };
        let type_definition_id = self.ir.type_definitions.push_return_idx(type_definition).into();

        let object = federated::Object {
            type_definition_id,
            implements_interfaces: Vec::new(),
            join_implements: Vec::new(),
            keys: Vec::new(),
            fields: federated::NO_FIELDS,
        };
        let id = federated::ObjectId::from(self.ir.objects.push_return_idx(object));
        self.ir
            .definitions_by_name
            .insert(name, federated::Definition::Object(id));

        id
    }

    pub(crate) fn insert_scalar(
        &mut self,
        scalar_name: &str,
        description: Option<&str>,
        composed_directives: federated::Directives,
    ) {
        let name = self.ir.strings.insert(scalar_name);
        let description = description.map(|description| self.ir.strings.insert(description));

        let scalar = federated::TypeDefinitionRecord {
            name,
            directives: composed_directives,
            description,
            kind: federated::TypeDefinitionKind::Scalar,
        };

        let id = self.ir.type_definitions.push_return_idx(scalar);
        self.ir
            .definitions_by_name
            .insert(name, federated::Definition::Scalar(id.into()));
    }

    pub(crate) fn insert_union(
        &mut self,
        name: federated::StringId,
        composed_directives: federated::Directives,
        description: Option<StringWalker<'_>>,
    ) -> federated::UnionId {
        let description = description.map(|description| self.ir.strings.insert(description.as_str()));

        let union = federated::Union {
            name,
            members: Vec::new(),
            join_members: BTreeSet::new(),
            composed_directives,
            description,
        };
        let id = federated::UnionId::from(self.ir.unions.push_return_idx(union));
        self.ir
            .definitions_by_name
            .insert(name, federated::Definition::Union(id));
        id
    }

    pub(crate) fn insert_union_member(
        &mut self,
        subgraph_id: SubgraphId,
        union_name: federated::StringId,
        member_name: federated::StringId,
    ) {
        self.ir.union_members.insert((union_name, member_name));

        match self.ir.union_join_members.entry((union_name, member_name)) {
            btree_map::Entry::Vacant(entry) => {
                entry.insert(vec![subgraph_id]);
            }
            btree_map::Entry::Occupied(mut entry) => {
                entry.get_mut().push(subgraph_id);
            }
        }
    }

    pub(crate) fn insert_key(&mut self, id: federated::ObjectId, key: subgraphs::KeyWalker<'_>) {
        self.ir.keys.push(ir::KeyIr {
            parent: federated::Definition::Object(id),
            key_id: key.id,
            is_interface_object: false,
            resolvable: key.is_resolvable(),
        });
    }

    pub(crate) fn insert_object_authorized(
        &mut self,
        object_id: federated::ObjectId,
        authorized_directive: subgraphs::DirectiveSiteId,
    ) {
        self.ir
            .object_authorized_directives
            .push((object_id, authorized_directive));
    }

    pub(crate) fn insert_interface_authorized(
        &mut self,
        interface_id: federated::InterfaceId,
        authorized_directive: subgraphs::DirectiveSiteId,
    ) {
        self.ir
            .interface_authorized_directives
            .push((interface_id, authorized_directive));
    }

    pub(crate) fn insert_string(&mut self, string_id: subgraphs::StringId) -> federated::StringId {
        self.ir.strings.insert(self.subgraphs.walk(string_id).as_str())
    }

    // We need a separate method for strings that appear in the federated graph but were not
    // interned in subgraphs.
    pub(crate) fn insert_static_str(&mut self, string: &'static str) -> federated::StringId {
        self.ir.strings.insert(string)
    }

    pub(crate) fn set_query(&mut self, id: federated::ObjectId) {
        self.ir.query_type = Some(id);
    }

    pub(crate) fn set_mutation(&mut self, id: federated::ObjectId) {
        self.ir.mutation_type = Some(id);
    }

    pub(crate) fn set_subscription(&mut self, id: federated::ObjectId) {
        self.ir.subscription_type = Some(id);
    }

    pub(crate) fn insert_object_field_from_entity_interface(
        &mut self,
        object_name: federated::StringId,
        field_id: federated::FieldId,
    ) {
        self.ir
            .object_fields_from_entity_interfaces
            .insert((object_name, field_id));
    }
}

use crate::{
    composition_ir::{self as ir, CompositionIr},
    subgraphs::{self, StringWalker},
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
            context.insert_scalar(builtin_scalar.as_str(), None, Vec::new());
        }

        context
    }

    pub(crate) fn into_ir(self) -> CompositionIr {
        self.ir
    }

    pub(crate) fn insert_enum(
        &mut self,
        enum_name: &str,
        description: Option<&str>,
        directives: Vec<ir::Directive>,
    ) -> federated::TypeDefinitionId {
        let name = self.ir.strings.insert(enum_name);
        let description = description.map(|description| self.ir.strings.insert(description));

        let r#enum = ir::TypeDefinitionIr {
            federated: federated::TypeDefinitionRecord {
                name,
                description,
                kind: federated::TypeDefinitionKind::Enum,
                // Populated when emitting the federated graph.
                directives: Vec::new(),
            },
            directives,
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
        directives: Vec<ir::Directive>,
        enum_id: federated::TypeDefinitionId,
    ) -> federated::EnumValueId {
        let value = self.ir.strings.insert(value);
        let description = description.map(|description| self.ir.strings.insert(description));

        federated::EnumValueId::from(self.ir.enum_values.push_return_idx(ir::EnumValueIr {
            federated: federated::EnumValueRecord {
                enum_id,
                value,
                description,
                // Populated when emitting the federated graph.
                directives: Vec::new(),
            },
            directives,
        }))
    }

    pub(crate) fn insert_field(&mut self, ir: ir::FieldIr) {
        self.ir.fields.push(ir);
    }

    pub(crate) fn insert_input_object(
        &mut self,
        name: federated::StringId,
        description: Option<&str>,
        directives: Vec<ir::Directive>,
        fields: federated::InputValueDefinitions,
    ) -> federated::InputObjectId {
        let description = description.map(|description| self.ir.strings.insert(description));
        let object = ir::InputObjectIr {
            federated: federated::InputObject {
                name,
                fields,
                description,
                // Populated when emitting the federated graph.
                directives: Vec::new(),
            },
            directives,
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
        directives: Vec<ir::Directive>,
    ) -> federated::InterfaceId {
        let description = description.map(|description| self.ir.strings.insert(description));

        let type_definition = ir::TypeDefinitionIr {
            federated: federated::TypeDefinitionRecord {
                name,
                description,
                kind: federated::TypeDefinitionKind::Interface,
                // Populated when emitting the federated graph.
                directives: Vec::new(),
            },
            directives,
        };
        let type_definition_id = self.ir.type_definitions.push_return_idx(type_definition).into();

        let interface = federated::Interface {
            type_definition_id,
            // Populated when emitting the federated graph.
            fields: federated::NO_FIELDS,
            implements_interfaces: Vec::new(),
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
        let id = self.ir.interfaces[usize::from(id)].type_definition_id;
        self.ir.type_definitions[usize::from(id)]
            .directives
            .push(ir::Directive::JoinType(ir::JoinTypeDirective {
                subgraph_id: federated::SubgraphId::from(key.parent_definition().subgraph_id().idx()),
                key: Some(key.id),
                is_interface_object,
            }))
    }

    pub(crate) fn insert_object(
        &mut self,
        name: federated::StringId,
        description: Option<StringWalker<'_>>,
        directives: Vec<ir::Directive>,
    ) -> federated::ObjectId {
        let description = description.map(|description| self.ir.strings.insert(description.as_str()));
        let type_definition = ir::TypeDefinitionIr {
            federated: federated::TypeDefinitionRecord {
                name,
                description,
                kind: federated::TypeDefinitionKind::Object,
                // Populated when emitting the federated graph.
                directives: Vec::new(),
            },
            directives,
        };
        let type_definition_id = self.ir.type_definitions.push_return_idx(type_definition).into();

        let object = federated::Object {
            type_definition_id,
            // Populated when emitting the federated graph.
            fields: federated::NO_FIELDS,
            implements_interfaces: Vec::new(),
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
        directives: Vec<ir::Directive>,
    ) {
        let name = self.ir.strings.insert(scalar_name);
        let description = description.map(|description| self.ir.strings.insert(description));

        let scalar = ir::TypeDefinitionIr {
            federated: federated::TypeDefinitionRecord {
                name,
                description,
                kind: federated::TypeDefinitionKind::Scalar,
                // Populated when emitting the federated graph.
                directives: Vec::new(),
            },
            directives,
        };

        let id = self.ir.type_definitions.push_return_idx(scalar);
        self.ir
            .definitions_by_name
            .insert(name, federated::Definition::Scalar(id.into()));
    }

    pub(crate) fn insert_union(
        &mut self,
        name: federated::StringId,
        directives: Vec<ir::Directive>,
        description: Option<StringWalker<'_>>,
    ) -> federated::UnionId {
        let description = description.map(|description| self.ir.strings.insert(description.as_str()));

        let union = ir::UnionIr {
            federated: federated::Union {
                name,
                description,
                // Populated when emitting the federated graph.
                members: Vec::new(),
                directives: Vec::new(),
            },
            directives,
        };
        let id = federated::UnionId::from(self.ir.unions.push_return_idx(union));
        self.ir
            .definitions_by_name
            .insert(name, federated::Definition::Union(id));
        id
    }

    pub(crate) fn insert_union_member(&mut self, union_name: federated::StringId, member_name: federated::StringId) {
        self.ir.union_members.insert((union_name, member_name));
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
}

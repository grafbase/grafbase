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
            context.insert_scalar(builtin_scalar, false, None);
        }

        context
    }

    pub(crate) fn has_query_type(&self) -> bool {
        self.ir.query_type.is_some()
    }

    pub(crate) fn into_ir(self) -> CompositionIr {
        self.ir
    }

    pub(crate) fn insert_enum(
        &mut self,
        enum_name: StringWalker<'_>,
        is_inaccessible: bool,
        description: Option<StringWalker<'_>>,
    ) -> federated::EnumId {
        let name = self.ir.insert_string(enum_name);
        let description = description.map(|description| self.ir.insert_string(description));

        let composed_directives = if is_inaccessible {
            vec![federated::Directive {
                name: self.insert_static_str("inaccessible"),
                arguments: Vec::new(),
            }]
        } else {
            Vec::new()
        };

        let r#enum = federated::Enum {
            name,
            values: Vec::new(),
            composed_directives,
            description,
        };
        let id = federated::EnumId(self.ir.enums.push_return_idx(r#enum));
        self.ir
            .definitions_by_name
            .insert(enum_name.id, federated::Definition::Enum(id));
        id
    }

    pub(crate) fn insert_enum_value(
        &mut self,
        enum_id: federated::EnumId,
        value: StringWalker<'_>,
        deprecation: Option<Option<StringWalker<'_>>>,
        description: Option<StringWalker<'_>>,
    ) {
        let mut composed_directives = Vec::new();

        if let Some(deprecation) = deprecation {
            let arguments = match deprecation {
                Some(reason) => vec![(
                    self.insert_static_str("reason"),
                    federated::Value::String(self.ir.insert_string(reason)),
                )],
                None => Vec::new(),
            };
            let name = self.insert_static_str("deprecated");

            composed_directives.push(federated::Directive { name, arguments });
        }

        let value = self.ir.insert_string(value);
        let description = description.map(|description| self.ir.insert_string(description));
        let r#enum = &mut self.ir.enums[enum_id.0];

        if r#enum.values.iter().any(|v| v.value == value) {
            return;
        }

        r#enum.values.push(federated::EnumValue {
            value,
            composed_directives,
            description,
        });
    }

    pub(crate) fn insert_field(&mut self, ir: ir::FieldIr) -> federated::FieldId {
        federated::FieldId(self.ir.fields.push_return_idx(ir))
    }

    pub(crate) fn insert_input_object(
        &mut self,
        input_object_name: StringWalker<'_>,
        is_inaccessible: bool,
        description: Option<StringWalker<'_>>,
    ) -> federated::InputObjectId {
        let name = self.ir.insert_string(input_object_name);
        let description = description.map(|description| self.ir.insert_string(description));
        let object = federated::InputObject {
            name,
            fields: Vec::new(),
            composed_directives: if is_inaccessible {
                vec![federated::Directive {
                    name: self.insert_static_str("inaccessible"),
                    arguments: Vec::new(),
                }]
            } else {
                Vec::new()
            },
            description,
        };
        let id = federated::InputObjectId(self.ir.input_objects.push_return_idx(object));
        self.ir
            .definitions_by_name
            .insert(input_object_name.id, federated::Definition::InputObject(id));
        id
    }

    pub(crate) fn insert_interface(
        &mut self,
        interface_name: StringWalker<'_>,
        is_inaccessible: bool,
        description: Option<StringWalker<'_>>,
    ) -> federated::InterfaceId {
        let name = self.ir.insert_string(interface_name);
        let description = description.map(|description| self.ir.insert_string(description));
        let mut composed_directives = Vec::new();

        if is_inaccessible {
            composed_directives.push(federated::Directive {
                name: self.insert_static_str("inaccessible"),
                arguments: Vec::new(),
            });
        }

        let interface = federated::Interface {
            name,
            implements_interfaces: Vec::new(),
            resolvable_keys: Vec::new(),
            composed_directives,
            description,
        };
        let id = federated::InterfaceId(self.ir.interfaces.push_return_idx(interface));
        self.ir
            .definitions_by_name
            .insert(interface_name.id, federated::Definition::Interface(id));
        id
    }

    pub(crate) fn insert_interface_resolvable_key(
        &mut self,
        id: federated::InterfaceId,
        key: subgraphs::KeyId,
        is_interface_object: bool,
    ) {
        self.ir
            .insert_resolvable_key(federated::Definition::Interface(id), key, is_interface_object);
    }

    pub(crate) fn insert_object(
        &mut self,
        object_name: StringWalker<'_>,
        description: Option<StringWalker<'_>>,
        composed_directives: Vec<federated::Directive>,
    ) -> federated::ObjectId {
        let name = self.ir.insert_string(object_name);
        let description = description.map(|description| self.ir.insert_string(description));

        let object = federated::Object {
            name,
            implements_interfaces: Vec::new(),
            resolvable_keys: Vec::new(),
            composed_directives,
            description,
        };
        let id = federated::ObjectId(self.ir.objects.push_return_idx(object));
        self.ir
            .definitions_by_name
            .insert(object_name.id, federated::Definition::Object(id));

        // FIXME: Those roots probably shouldn't be hardcoded.
        match object_name.as_str() {
            "Query" => self.ir.query_type = Some(id),
            "Mutation" => self.ir.mutation_type = Some(id),
            "Subscription" => self.ir.subscription_type = Some(id),
            _ => (),
        }

        id
    }

    pub(crate) fn insert_scalar(
        &mut self,
        scalar_name: StringWalker<'_>,
        is_inaccessible: bool,
        description: Option<StringWalker<'_>>,
    ) {
        let name = self.ir.insert_string(scalar_name);
        let description = description.map(|description| self.ir.insert_string(description));
        let mut composed_directives = Vec::new();

        if is_inaccessible {
            composed_directives.push(federated::Directive {
                name: self.insert_static_str("inaccessible"),
                arguments: Vec::new(),
            });
        }

        let scalar = federated::Scalar {
            name,
            composed_directives,
            description,
        };

        let id = federated::ScalarId(self.ir.scalars.push_return_idx(scalar));
        self.ir
            .definitions_by_name
            .insert(scalar_name.id, federated::Definition::Scalar(id));
    }

    pub(crate) fn insert_union(
        &mut self,
        union_name: StringWalker<'_>,
        is_inaccessible: bool,
        description: Option<StringWalker<'_>>,
    ) -> federated::UnionId {
        let name = self.ir.insert_string(union_name);
        let description = description.map(|description| self.ir.insert_string(description));

        let composed_directives = if is_inaccessible {
            vec![federated::Directive {
                name: self.insert_static_str("inaccessible"),
                arguments: Vec::new(),
            }]
        } else {
            Vec::new()
        };
        let union = federated::Union {
            name,
            members: Vec::new(),
            composed_directives,
            description,
        };
        let id = federated::UnionId(self.ir.unions.push_return_idx(union));
        self.ir
            .definitions_by_name
            .insert(union_name.id, federated::Definition::Union(id));
        id
    }

    pub(crate) fn insert_union_member(&mut self, union_name: subgraphs::StringId, member_name: subgraphs::StringId) {
        self.ir.insert_union_member(union_name, member_name);
    }

    pub(crate) fn insert_resolvable_key(&mut self, object_id: federated::ObjectId, key_id: subgraphs::KeyId) {
        self.ir
            .insert_resolvable_key(federated::Definition::Object(object_id), key_id, false);
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
}

impl Context<'_> {}

use crate::{
    subgraphs::{self, StringWalker},
    VecExt,
};
use graphql_federated_graph as federated;
use std::collections::{BTreeSet, HashMap};

/// The intermediate representation of the schema that is produced by composition. This data
/// structure is distinct from [FederatedGraph](graphql_federated_graph::FederatedGraph) because
/// it is accumulated out of order during composition. Only after all composed top-level
/// definitions (objects, interfaces, scalars, enums, unions) are known can we construct
/// [federated::FieldTypeId]s and [federated::ObjectId]s for union members for the federated graph.
///
/// This is a **write only** data structure during composition. The source of truth for the
/// contents of the federated graph is the subgraphs.
#[derive(Default)]
pub(crate) struct CompositionIr {
    pub(crate) definitions_by_name: HashMap<subgraphs::StringId, federated::Definition>,

    pub(crate) objects: Vec<federated::Object>,
    pub(crate) interfaces: Vec<federated::Interface>,
    pub(crate) unions: Vec<federated::Union>,
    pub(crate) enums: Vec<federated::Enum>,
    pub(crate) scalars: Vec<federated::Scalar>,
    pub(crate) input_objects: Vec<federated::InputObject>,

    /// The root `Query` type
    pub(crate) query_type: Option<federated::ObjectId>,

    /// The root `Mutation` type
    pub(crate) mutation_type: Option<federated::ObjectId>,

    /// The root `Subscription` type
    pub(crate) subscription_type: Option<federated::ObjectId>,

    pub(crate) strings: StringsIr,
    pub(crate) fields: Vec<FieldIr>,
    pub(crate) union_members: BTreeSet<(subgraphs::StringId, subgraphs::StringId)>,
    pub(crate) resolvable_keys: Vec<KeyIr>,
}

impl CompositionIr {
    pub(crate) fn insert_enum(&mut self, enum_name: StringWalker<'_>) -> federated::EnumId {
        let name = self.insert_string(enum_name);
        let r#enum = federated::Enum {
            name,
            values: Vec::new(),
            composed_directives: Vec::new(),
        };
        let id = federated::EnumId(self.enums.push_return_idx(r#enum));
        self.definitions_by_name
            .insert(enum_name.id, federated::Definition::Enum(id));
        id
    }

    pub(crate) fn insert_interface(&mut self, iface_name: StringWalker<'_>) -> federated::InterfaceId {
        let name = self.insert_string(iface_name);
        let iface = federated::Interface {
            name,
            implements_interfaces: Vec::new(),
            composed_directives: Vec::new(),
        };
        let id = federated::InterfaceId(self.interfaces.push_return_idx(iface));
        self.definitions_by_name
            .insert(iface_name.id, federated::Definition::Interface(id));
        id
    }

    pub(crate) fn insert_scalar(&mut self, scalar_name: StringWalker<'_>) {
        let name = self.insert_string(scalar_name);
        let scalar = federated::Scalar {
            name,
            composed_directives: Vec::new(),
        };
        let id = federated::ScalarId(self.scalars.push_return_idx(scalar));
        self.definitions_by_name
            .insert(scalar_name.id, federated::Definition::Scalar(id));
    }

    pub(crate) fn insert_input_object(&mut self, input_object_name: StringWalker<'_>) -> federated::InputObjectId {
        let name = self.insert_string(input_object_name);
        let object = federated::InputObject {
            name,
            fields: Vec::new(),
            composed_directives: Vec::new(),
        };
        let id = federated::InputObjectId(self.input_objects.push_return_idx(object));
        self.definitions_by_name
            .insert(input_object_name.id, federated::Definition::InputObject(id));
        id
    }

    pub(crate) fn insert_resolvable_key(&mut self, object_id: federated::ObjectId, key_id: subgraphs::KeyId) {
        self.resolvable_keys.push(KeyIr { object_id, key_id });
    }

    pub(crate) fn insert_object(
        &mut self,
        object_name: StringWalker<'_>,
        is_inaccessible: bool,
    ) -> federated::ObjectId {
        let name = self.insert_string(object_name);
        let mut composed_directives = Vec::new();
        if is_inaccessible {
            composed_directives.push(federated::Directive {
                name: self.insert_static_str("inaccessible"),
                arguments: Vec::new(),
            });
        }
        let object = federated::Object {
            name,
            implements_interfaces: Vec::new(),
            resolvable_keys: Vec::new(),
            composed_directives,
        };
        let id = federated::ObjectId(self.objects.push_return_idx(object));
        self.definitions_by_name
            .insert(object_name.id, federated::Definition::Object(id));

        // FIXME: Those roots probably shouldn't be hardcoded.
        match object_name.as_str() {
            "Query" => self.query_type = Some(id),
            "Mutation" => self.mutation_type = Some(id),
            "Subscription" => self.subscription_type = Some(id),
            _ => (),
        }

        id
    }

    pub(crate) fn insert_union(&mut self, union_name: StringWalker<'_>) -> federated::UnionId {
        let name = self.insert_string(union_name);
        let union = federated::Union {
            name,
            members: Vec::new(),
            composed_directives: Vec::new(),
        };
        let id = federated::UnionId(self.unions.push_return_idx(union));
        self.definitions_by_name
            .insert(union_name.id, federated::Definition::Union(id));
        id
    }

    pub(crate) fn insert_enum_value(
        &mut self,
        enum_id: federated::EnumId,
        value: StringWalker<'_>,
        deprecation: Option<Option<StringWalker<'_>>>,
    ) {
        let mut composed_directives = Vec::new();

        if let Some(deprecation) = deprecation {
            let arguments = match deprecation {
                Some(reason) => vec![(
                    self.insert_static_str("reason"),
                    federated::Value::String(self.insert_string(reason)),
                )],
                None => Vec::new(),
            };
            let name = self.insert_static_str("deprecated");

            composed_directives.push(federated::Directive { name, arguments });
        }

        let value = self.insert_string(value);
        let r#enum = &mut self.enums[enum_id.0];

        if r#enum.values.iter().any(|v| v.value == value) {
            return;
        }

        r#enum.values.push(federated::EnumValue {
            value,
            composed_directives,
        });
    }

    pub(crate) fn insert_field(&mut self, ir: FieldIr) {
        self.fields.push(ir);
    }

    pub(crate) fn insert_union_member(&mut self, union_name: subgraphs::StringId, member_name: subgraphs::StringId) {
        self.union_members.insert((union_name, member_name));
    }

    pub(crate) fn insert_string(&mut self, string: subgraphs::StringWalker<'_>) -> federated::StringId {
        self.strings.insert(string)
    }

    pub(crate) fn insert_static_str(&mut self, string: &'static str) -> federated::StringId {
        self.strings.insert_static_str(string)
    }
}

pub(crate) struct FieldIr {
    pub(crate) parent_name: subgraphs::StringId,
    pub(crate) field_name: subgraphs::StringId,
    pub(crate) field_type: subgraphs::FieldTypeId,
    pub(crate) arguments: Vec<(subgraphs::StringId, subgraphs::FieldTypeId)>,
    pub(crate) resolvable_in: Option<federated::SubgraphId>,

    /// Subgraph fields with an `@provides`.
    pub(crate) provides: Vec<subgraphs::FieldId>,

    /// Subgraph fields with an `@requires`.
    pub(crate) requires: Vec<subgraphs::FieldId>,

    pub(crate) composed_directives: Vec<federated::Directive>,
}

#[derive(Default)]
pub(crate) struct StringsIr {
    pub(crate) map: HashMap<subgraphs::StringId, federated::StringId>,
    pub(crate) static_str_map: HashMap<&'static str, federated::StringId>,
    pub(crate) strings: Vec<String>,
}

impl StringsIr {
    pub(crate) fn insert_static_str(&mut self, string: &'static str) -> federated::StringId {
        *self
            .static_str_map
            .entry(string)
            .or_insert_with(|| federated::StringId(self.strings.push_return_idx(string.to_owned())))
    }

    pub(crate) fn insert(&mut self, string: subgraphs::StringWalker<'_>) -> federated::StringId {
        *self
            .map
            .entry(string.id)
            .or_insert_with(|| federated::StringId(self.strings.push_return_idx(string.as_str().to_owned())))
    }
}

pub(crate) struct KeyIr {
    pub(crate) object_id: federated::ObjectId,
    pub(crate) key_id: subgraphs::KeyId,
}

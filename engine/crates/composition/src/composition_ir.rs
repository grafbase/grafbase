mod directive;

pub(crate) use self::directive::Directive;
use crate::subgraphs::{self, SubgraphId};
use graphql_federated_graph as federated;
use std::collections::{BTreeMap, BTreeSet, HashMap};

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
    pub(crate) definitions_by_name: HashMap<federated::StringId, federated::Definition>,

    pub(crate) type_definitions: Vec<federated::TypeDefinitionRecord>,
    pub(crate) objects: Vec<federated::Object>,
    pub(crate) interfaces: Vec<federated::Interface>,
    pub(crate) unions: Vec<federated::Union>,
    pub(crate) enum_values: Vec<federated::EnumValueRecord>,
    pub(crate) input_objects: Vec<federated::InputObject>,
    pub(crate) directives: Vec<Directive>,
    pub(crate) input_value_definitions: Vec<InputValueDefinitionIr>,

    /// The root `Query` type
    pub(crate) query_type: Option<federated::ObjectId>,

    /// The root `Mutation` type
    pub(crate) mutation_type: Option<federated::ObjectId>,

    /// The root `Subscription` type
    pub(crate) subscription_type: Option<federated::ObjectId>,

    pub(crate) strings: StringsIr,
    pub(crate) fields: Vec<FieldIr>,
    pub(crate) union_members: BTreeSet<(federated::StringId, federated::StringId)>,
    pub(crate) union_join_members: BTreeMap<(federated::StringId, federated::StringId), Vec<SubgraphId>>,
    pub(crate) keys: Vec<KeyIr>,

    /// Fields of an interface entity that are contributed by other subgraphs and must be added to
    /// the interface's implementers in the federated schema
    pub(crate) object_fields_from_entity_interfaces: BTreeSet<(federated::StringId, federated::FieldId)>,

    /// @authorized directives on objects
    pub(crate) object_authorized_directives: Vec<(federated::ObjectId, subgraphs::DirectiveSiteId)>,
    /// @authorized directives on interfaces
    pub(crate) interface_authorized_directives: Vec<(federated::InterfaceId, subgraphs::DirectiveSiteId)>,
}

#[derive(Clone)]
pub(crate) struct FieldIr {
    pub(crate) parent_definition: federated::Definition,
    pub(crate) field_name: federated::StringId,
    pub(crate) field_type: subgraphs::FieldTypeId,
    pub(crate) arguments: federated::InputValueDefinitions,

    pub(crate) resolvable_in: Vec<federated::SubgraphId>,

    /// Subgraph fields corresponding to this federated graph field that have an `@provides`.
    pub(crate) provides: Vec<subgraphs::FieldId>,

    /// Subgraph fields corresponding to this federated graph field that have an `@requires`.
    pub(crate) requires: Vec<subgraphs::FieldId>,

    /// Subgraph fields corresponding to this federated graph field that have an `@authorized`.
    pub(crate) authorized_directives: Vec<subgraphs::FieldId>,

    // @join__field(graph: ..., override: ...)
    pub(crate) overrides: Vec<federated::Override>,

    pub(crate) composed_directives: federated::Directives,

    pub(crate) description: Option<federated::StringId>,
}

#[derive(Clone)]
pub(crate) struct InputValueDefinitionIr {
    pub(crate) name: federated::StringId,
    pub(crate) r#type: subgraphs::FieldTypeId,
    pub(crate) directives: federated::Directives,
    pub(crate) description: Option<federated::StringId>,
    pub(crate) default: Option<subgraphs::Value>,
}

#[derive(Default)]
pub(crate) struct StringsIr {
    strings: indexmap::IndexSet<String>,
}

impl StringsIr {
    pub(crate) fn insert(&mut self, string: &str) -> federated::StringId {
        let idx = self
            .strings
            .get_index_of(string)
            .unwrap_or_else(|| self.strings.insert_full(string.to_owned()).0);

        federated::StringId(idx)
    }

    pub(crate) fn into_federated_strings(self) -> Vec<String> {
        self.strings.into_iter().collect()
    }
}

impl std::ops::Index<federated::StringId> for StringsIr {
    type Output = str;

    fn index(&self, index: federated::StringId) -> &Self::Output {
        self.strings.get_index(index.0).unwrap().as_str()
    }
}

pub(crate) struct KeyIr {
    pub(crate) parent: federated::Definition,
    pub(crate) key_id: subgraphs::KeyId,
    pub(crate) is_interface_object: bool,
    pub(crate) resolvable: bool,
}

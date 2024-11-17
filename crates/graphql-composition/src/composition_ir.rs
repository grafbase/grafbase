mod directive;
mod field_ir;

pub(crate) use self::{directive::Directive, field_ir::*};
use crate::subgraphs::{self, SubgraphId};
use graphql_federated_graph::{self as federated, directives::ListSizeDirective};
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

    /// @authorized directives on objects
    pub(crate) object_authorized_directives: Vec<(federated::ObjectId, subgraphs::DirectiveSiteId)>,
    /// @authorized directives on interfaces
    pub(crate) interface_authorized_directives: Vec<(federated::InterfaceId, subgraphs::DirectiveSiteId)>,

    // @listSize on fields
    //
    // Indexed by (definition_name, field_name) because we dont have stable id for fields
    // at the point this is constructed.
    //
    // These are separate from FieldIr because they need to reference fields on other types
    // so should be constructed last.
    pub(crate) list_sizes: BTreeMap<(federated::StringId, federated::StringId), ListSizeDirective>,
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

        federated::StringId::from(idx)
    }

    pub(crate) fn lookup(&self, string: &str) -> Option<federated::StringId> {
        Some(federated::StringId::from(self.strings.get_index_of(string)?))
    }

    pub(crate) fn into_federated_strings(self) -> Vec<String> {
        self.strings.into_iter().collect()
    }
}

impl std::ops::Index<federated::StringId> for StringsIr {
    type Output = str;

    fn index(&self, index: federated::StringId) -> &Self::Output {
        self.strings.get_index(usize::from(index)).unwrap().as_str()
    }
}

pub(crate) struct KeyIr {
    pub(crate) parent: federated::Definition,
    pub(crate) key_id: subgraphs::KeyId,
    pub(crate) is_interface_object: bool,
    pub(crate) resolvable: bool,
}

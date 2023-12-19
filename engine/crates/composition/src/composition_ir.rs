use crate::{subgraphs, VecExt};
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
    pub(crate) definitions_by_name: HashMap<federated::StringId, federated::Definition>,

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
    pub(crate) union_members: BTreeSet<(federated::StringId, federated::StringId)>,
    pub(crate) resolvable_keys: Vec<KeyIr>,

    /// Fields on implementers of an interface entity's implementers that are contributed by other
    /// subgraphs.
    pub(crate) object_fields_from_entity_interfaces: BTreeSet<(federated::StringId, federated::FieldId)>,
}

impl CompositionIr {
    pub(crate) fn insert_resolvable_key(
        &mut self,
        parent: federated::Definition,
        key_id: subgraphs::KeyId,
        is_interface_object: bool,
    ) {
        self.resolvable_keys.push(KeyIr {
            parent,
            key_id,
            is_interface_object,
        });
    }

    pub(crate) fn insert_string(&mut self, string: subgraphs::StringWalker<'_>) -> federated::StringId {
        self.strings.insert(string)
    }

    pub(crate) fn insert_static_str(&mut self, string: &'static str) -> federated::StringId {
        self.strings.insert_static_str(string)
    }
}

pub(crate) struct FieldIr {
    pub(crate) parent_definition: federated::Definition,
    pub(crate) field_name: subgraphs::StringId,
    pub(crate) field_type: subgraphs::FieldTypeId,
    pub(crate) arguments: Vec<ArgumentIr>,

    pub(crate) resolvable_in: Option<federated::SubgraphId>,

    /// Subgraph fields corresponding to this federated graph field that have an `@provides`.
    pub(crate) provides: Vec<subgraphs::FieldId>,

    /// Subgraph fields corresponding to this federated graph field that have an `@requires`.
    pub(crate) requires: Vec<subgraphs::FieldId>,

    // @join__field(graph: ..., override: ...)
    pub(crate) overrides: Vec<federated::Override>,

    pub(crate) composed_directives: Vec<federated::Directive>,

    pub(crate) description: Option<federated::StringId>,
}

#[derive(Clone)]
pub(crate) struct ArgumentIr {
    pub(crate) argument_name: subgraphs::StringId,
    pub(crate) argument_type: subgraphs::FieldTypeId,
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
    pub(crate) parent: federated::Definition,
    pub(crate) key_id: subgraphs::KeyId,
    pub(crate) is_interface_object: bool,
}

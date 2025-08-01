mod directive;
mod directive_definition;
mod field_ir;

pub(crate) use self::{directive::*, directive_definition::*, field_ir::*};

use crate::{federated_graph as federated, subgraphs};
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

    pub(crate) scalar_definitions: Vec<(federated::ScalarDefinitionRecord, Vec<Directive>)>,
    pub(crate) enum_definitions: Vec<(federated::EnumDefinitionRecord, Vec<Directive>)>,
    pub(crate) objects: Vec<(federated::Object, Vec<Directive>)>,
    pub(crate) interfaces: Vec<(federated::Interface, Vec<Directive>)>,
    pub(crate) unions: Vec<UnionIr>,
    pub(crate) enum_values: Vec<EnumValueIr>,
    pub(crate) input_objects: Vec<InputObjectIr>,
    pub(crate) input_value_definitions: Vec<InputValueDefinitionIr>,
    pub(crate) directive_definitions: Vec<DirectiveDefinitionIr>,

    /// The root `Query` type
    pub(crate) query_type: Option<federated::ObjectId>,

    /// The root `Mutation` type
    pub(crate) mutation_type: Option<federated::ObjectId>,

    /// The root `Subscription` type
    pub(crate) subscription_type: Option<federated::ObjectId>,

    pub(crate) strings: StringsIr,
    pub(crate) fields: Vec<FieldIr>,
    pub(crate) union_members: BTreeSet<(federated::StringId, federated::StringId)>,

    // indexed by ExtensionId
    pub(crate) used_extensions: fixedbitset::FixedBitSet,
}

pub(crate) struct InputObjectIr {
    pub federated: federated::InputObject,
    pub directives: Vec<Directive>,
}

pub(crate) struct EnumValueIr {
    pub federated: federated::EnumValueRecord,
    pub directives: Vec<Directive>,
}

pub(crate) struct UnionIr {
    pub federated: federated::Union,
    pub directives: Vec<Directive>,
}

pub(crate) struct InputValueDefinitionIr {
    pub(crate) name: federated::StringId,
    pub(crate) r#type: subgraphs::FieldType,
    pub(crate) directives: Vec<Directive>,
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

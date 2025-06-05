use crate::{
    federated_graph::{self as federated, OverrideLabel, OverrideSource},
    subgraphs::{self, FieldId, FieldTuple, FieldType, KeyId},
};

#[derive(Clone, PartialEq)]
pub enum Directive {
    Authenticated,
    OneOf,
    Deprecated {
        reason: Option<federated::StringId>,
    },
    Inaccessible,
    Policy(Vec<Vec<federated::StringId>>),
    RequiresScopes(Vec<Vec<federated::StringId>>),
    /// @composite__require
    CompositeRequire {
        subgraph_id: federated::SubgraphId,
        field: subgraphs::StringId,
    },
    /// @composite__is
    CompositeIs {
        subgraph_id: federated::SubgraphId,
        field: subgraphs::StringId,
    },
    /// @composite__internal
    CompositeInternal(federated::SubgraphId),
    /// @composite__lookup
    CompositeLookup(federated::SubgraphId),
    /// @composite__derive
    CompositeDerive(federated::SubgraphId),
    Cost {
        weight: i32,
    },
    Other {
        name: federated::StringId,
        arguments: Vec<(federated::StringId, subgraphs::Value)>,
        provenance: DirectiveProvenance,
    },
    JoinField(JoinFieldDirective),
    JoinEntityInterfaceField,
    JoinInputField(JoinInputFieldDirective),
    JoinType(JoinTypeDirective),
    ListSize(federated::ListSizeDirective),
    JoinUnionMember(JoinUnionMemberDirective),
}

#[derive(PartialEq, PartialOrd, Clone)]
pub struct JoinUnionMemberDirective {
    pub member: subgraphs::DefinitionId,
}

#[derive(PartialEq, PartialOrd, Clone)]
pub struct JoinInputFieldDirective {
    pub subgraph_id: federated::SubgraphId,
    pub r#type: Option<FieldType>,
}

#[derive(PartialEq, PartialOrd, Clone)]
pub struct JoinFieldDirective {
    pub source_field: (FieldId, FieldTuple),
    pub r#type: Option<FieldType>,
    pub external: bool,
    pub r#override: Option<OverrideSource>,
    pub override_label: Option<OverrideLabel>,
}

#[derive(PartialEq, PartialOrd, Clone)]
pub struct JoinTypeDirective {
    pub subgraph_id: federated::SubgraphId,
    pub key: Option<KeyId>,
    pub is_interface_object: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum DirectiveProvenance {
    LinkedFromExtension {
        linked_schema_id: subgraphs::LinkedSchemaId,
        extension_id: subgraphs::ExtensionId,
    },
    ComposeDirective,
    Builtin,
}

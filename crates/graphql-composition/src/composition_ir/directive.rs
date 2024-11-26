use crate::subgraphs::{self, DirectiveSiteId, FieldId, FieldTuple, FieldTypeId, KeyId};
use graphql_federated_graph::{self as federated, OverrideLabel, OverrideSource};

#[derive(PartialEq, PartialOrd, Clone)]
pub enum Directive {
    Authenticated,
    Deprecated {
        reason: Option<federated::StringId>,
    },
    Inaccessible,
    Policy(Vec<Vec<federated::StringId>>),
    RequiresScopes(Vec<Vec<federated::StringId>>),
    Cost {
        weight: i32,
    },
    Other {
        name: federated::StringId,
        arguments: Vec<(federated::StringId, subgraphs::Value)>,
    },
    JoinField(JoinFieldDirective),
    JoinInputField(JoinInputFieldDirective),
    Authorized(AuthorizedDirective),
    JoinType(JoinTypeDirective),
    ListSize(federated::ListSizeDirective),
    JoinUnionMember(JoinUnionMemberDirective),
}

#[derive(PartialEq, PartialOrd, Clone)]
pub struct AuthorizedDirective {
    pub source: DirectiveSiteId,
}

#[derive(PartialEq, PartialOrd, Clone)]
pub struct JoinUnionMemberDirective {
    pub member: subgraphs::DefinitionId,
}

#[derive(PartialEq, PartialOrd, Clone)]
pub struct JoinInputFieldDirective {
    pub subgraph_id: federated::SubgraphId,
    pub r#type: Option<FieldTypeId>,
}

#[derive(PartialEq, PartialOrd, Clone)]
pub struct JoinFieldDirective {
    pub source_field: (FieldId, FieldTuple),
    pub r#override: Option<OverrideSource>,
    pub override_label: Option<OverrideLabel>,
    pub r#type: Option<FieldTypeId>,
}

#[derive(PartialEq, PartialOrd, Clone)]
pub struct JoinTypeDirective {
    pub subgraph_id: federated::SubgraphId,
    pub key: Option<KeyId>,
    pub is_interface_object: bool,
}

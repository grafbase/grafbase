use crate::subgraphs;
use graphql_federated_graph::{self as federated, directives::ListSizeDirective};

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
    ListSize(ListSizeDirective),

    Other {
        name: federated::StringId,
        arguments: Vec<(federated::StringId, subgraphs::Value)>,
    },
}

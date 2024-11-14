use crate::subgraphs;
use graphql_federated_graph::{self as federated};

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
}

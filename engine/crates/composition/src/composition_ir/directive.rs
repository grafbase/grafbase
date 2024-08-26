use crate::subgraphs;
use graphql_federated_graph as federated;

#[derive(PartialEq, PartialOrd, Clone)]
pub enum Directive {
    Authenticated,
    Deprecated {
        reason: Option<federated::StringId>,
    },
    Inaccessible,
    Policy(Vec<Vec<federated::StringId>>),
    RequiresScopes(Vec<Vec<federated::StringId>>),

    Other {
        name: federated::StringId,
        arguments: Vec<(federated::StringId, subgraphs::Value)>,
    },
}

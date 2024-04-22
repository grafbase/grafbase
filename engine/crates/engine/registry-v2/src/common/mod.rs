mod directive_locations;
mod id_range;
mod iter;
mod types;

pub use directive_locations::DirectiveLocation;
pub use id_range::{IdOperations, IdRange};
pub use iter::*;
pub use types::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

impl OperationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationType::Query => "query",
            OperationType::Mutation => "mutation",
            OperationType::Subscription => "subscription",
        }
    }
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

mod v3;

use std::collections::HashMap;

use petgraph::{graph::NodeIndex, Graph};

use crate::{
    graph::{Edge, Node},
    Error,
};

pub use v3::parse;

#[derive(Default)]
pub struct Context {
    pub graph: Graph<Node, Edge>,
    schema_index: HashMap<Ref, NodeIndex>,
    pub operation_indices: Vec<NodeIndex>,
    errors: Vec<Error>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ref(pub(self) String);

impl std::fmt::Display for Ref {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

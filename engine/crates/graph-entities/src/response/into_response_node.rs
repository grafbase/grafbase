use super::QueryResponseNode;
use crate::{CompactValue, ResponseContainer, ResponseList, ResponsePrimitive};

/// Converts things into a QueryResponseNode
///
/// Implementations are defined for all the different node types
pub trait IntoResponseNode {
    /// Converts self into a QueryResponseNode
    fn into_node(self) -> QueryResponseNode;
}

impl IntoResponseNode for Box<ResponsePrimitive> {
    fn into_node(self) -> QueryResponseNode {
        QueryResponseNode::Primitive(self)
    }
}

impl IntoResponseNode for Box<ResponseList> {
    fn into_node(self) -> QueryResponseNode {
        crate::QueryResponseNode::List(self)
    }
}

impl IntoResponseNode for ResponseContainer {
    fn into_node(self) -> QueryResponseNode {
        QueryResponseNode::Container(Box::new(self))
    }
}

impl IntoResponseNode for CompactValue {
    fn into_node(self) -> QueryResponseNode {
        QueryResponseNode::Primitive(ResponsePrimitive::new(self))
    }
}

use crate::{CompactValue, ResponseContainer, ResponseList, ResponsePrimitive};
use internment::ArcIntern;

use super::QueryResponseNode;

/// Converts things into a QueryResponseNode
///
/// Implementations are defined for all the different node types
pub trait IntoResponseNode {
    /// The `NodeId` of this node in the database (if it has one)
    ///
    /// Not to be confused with the ResponseNodeId
    fn id(&self) -> Option<ArcIntern<String>>;
    /// Converts self into a QueryResponseNode
    fn into_node(self) -> QueryResponseNode;
}

impl IntoResponseNode for Box<ResponsePrimitive> {
    fn id(&self) -> Option<ArcIntern<String>> {
        None
    }

    fn into_node(self) -> QueryResponseNode {
        QueryResponseNode::Primitive(self)
    }
}

impl IntoResponseNode for Box<ResponseList> {
    fn id(&self) -> Option<ArcIntern<String>> {
        None
    }

    fn into_node(self) -> QueryResponseNode {
        crate::QueryResponseNode::List(self)
    }
}

impl IntoResponseNode for ResponseContainer {
    fn id(&self) -> Option<ArcIntern<String>> {
        self.id.clone()
    }

    fn into_node(self) -> QueryResponseNode {
        QueryResponseNode::Container(Box::new(self))
    }
}

impl IntoResponseNode for CompactValue {
    fn id(&self) -> Option<ArcIntern<String>> {
        None
    }

    fn into_node(self) -> QueryResponseNode {
        QueryResponseNode::Primitive(ResponsePrimitive::new(self))
    }
}

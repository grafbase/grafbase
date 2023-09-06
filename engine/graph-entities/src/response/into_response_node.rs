use super::{EntityId, QueryResponseNode};
use crate::{CompactValue, ResponseContainer, ResponseList, ResponsePrimitive};

/// Converts things into a QueryResponseNode
///
/// Implementations are defined for all the different node types
pub trait IntoResponseNode {
    /// The `EntityId` for this node if it has one
    ///
    /// Not to be confused with the ResponseNodeId
    fn entity_id(&self) -> Option<EntityId>;
    /// Converts self into a QueryResponseNode
    fn into_node(self) -> QueryResponseNode;
}

impl IntoResponseNode for Box<ResponsePrimitive> {
    fn entity_id(&self) -> Option<EntityId> {
        None
    }

    fn into_node(self) -> QueryResponseNode {
        QueryResponseNode::Primitive(self)
    }
}

impl IntoResponseNode for Box<ResponseList> {
    fn entity_id(&self) -> Option<EntityId> {
        None
    }

    fn into_node(self) -> QueryResponseNode {
        crate::QueryResponseNode::List(self)
    }
}

impl IntoResponseNode for ResponseContainer {
    fn entity_id(&self) -> Option<EntityId> {
        self.id.clone()
    }

    fn into_node(self) -> QueryResponseNode {
        QueryResponseNode::Container(Box::new(self))
    }
}

impl IntoResponseNode for CompactValue {
    fn entity_id(&self) -> Option<EntityId> {
        None
    }

    fn into_node(self) -> QueryResponseNode {
        QueryResponseNode::Primitive(ResponsePrimitive::new(self))
    }
}

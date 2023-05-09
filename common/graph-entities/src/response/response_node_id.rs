use crate::NodeID;
use internment::ArcIntern;
use serde::{Deserialize, Serialize};

use super::EntityId;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, Serialize, Deserialize)]
/// The identifier of a node within a [`QueryResponse`].
///
/// This ID is used to lookup other nodes in the graph and to link
/// a node to its children
pub struct ResponseNodeId(pub(crate) u32);

/// A type that can look up a ResponseNode inside a Response.
pub trait ResponseNodeReference {
    /// The EntityId if this type has one
    fn entity_id(&self) -> Option<EntityId>;
    /// The id of this node in the response if it has one
    fn response_node_id(&self, response: &super::QueryResponse) -> Option<ResponseNodeId>;
}

impl ResponseNodeReference for ResponseNodeId {
    fn entity_id(&self) -> Option<EntityId> {
        None
    }

    fn response_node_id(&self, _response: &super::QueryResponse) -> Option<ResponseNodeId> {
        Some(*self)
    }
}

impl ResponseNodeReference for NodeID<'_> {
    fn entity_id(&self) -> Option<EntityId> {
        Some(self.clone().into())
    }

    fn response_node_id(&self, response: &super::QueryResponse) -> Option<ResponseNodeId> {
        response.entity_ids.get(&self.entity_id().unwrap()).copied()
    }
}

impl ResponseNodeReference for EntityId {
    fn entity_id(&self) -> Option<EntityId> {
        Some(self.clone())
    }

    fn response_node_id(&self, response: &super::QueryResponse) -> Option<ResponseNodeId> {
        response.entity_ids.get(self).copied()
    }
}

impl ResponseNodeReference for ArcIntern<String> {
    fn entity_id(&self) -> Option<EntityId> {
        Some(self.clone().into())
    }

    fn response_node_id(&self, response: &super::QueryResponse) -> Option<ResponseNodeId> {
        response.entity_ids.get(&self.entity_id().unwrap()).copied()
    }
}

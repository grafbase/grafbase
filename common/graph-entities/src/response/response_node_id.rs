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

/// An abstraction around the various IDs we need to use with a QueryResponse
///
/// Internal to the QueryResponse we use a ResponseNodeId, but code that's
/// interacting with a QueryResponse might only know an entity by its `NodeId`.
///
/// The QueryResponse uses this trait to allow that code to just pass in a
/// `NodeId` (or the `ArcIntern<String>` version of a `NodeId`).
pub trait ResponseIdLookup {
    /// The EntityId if this type has one
    fn entity_id(&self) -> Option<EntityId>;
    /// The id of this node in the response if it has one
    fn response_node_id(&self, response: &super::QueryResponse) -> Option<ResponseNodeId>;
}

impl ResponseIdLookup for ResponseNodeId {
    fn entity_id(&self) -> Option<EntityId> {
        None
    }

    fn response_node_id(&self, _response: &super::QueryResponse) -> Option<ResponseNodeId> {
        Some(*self)
    }
}

impl ResponseIdLookup for NodeID<'_> {
    fn entity_id(&self) -> Option<EntityId> {
        Some(self.clone().into())
    }

    fn response_node_id(&self, response: &super::QueryResponse) -> Option<ResponseNodeId> {
        response.entity_ids.get(&self.entity_id().unwrap()).copied()
    }
}

impl ResponseIdLookup for EntityId {
    fn entity_id(&self) -> Option<EntityId> {
        Some(self.clone())
    }

    fn response_node_id(&self, response: &super::QueryResponse) -> Option<ResponseNodeId> {
        response.entity_ids.get(self).copied()
    }
}

impl ResponseIdLookup for ArcIntern<String> {
    fn entity_id(&self) -> Option<EntityId> {
        Some(self.clone().into())
    }

    fn response_node_id(&self, response: &super::QueryResponse) -> Option<ResponseNodeId> {
        response.entity_ids.get(&self.entity_id().unwrap()).copied()
    }
}

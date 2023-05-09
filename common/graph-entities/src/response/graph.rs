use crate::NodeID;
use internment::ArcIntern;
use serde::{Deserialize, Serialize};

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
    /// The database `NodeId` if it has one
    fn node_id(&self) -> Option<ArcIntern<String>>;
    /// The id of this node in the response if it has one
    fn response_node_id(&self, response: &super::QueryResponse) -> Option<ResponseNodeId>;
}

impl ResponseIdLookup for ResponseNodeId {
    fn node_id(&self) -> Option<ArcIntern<String>> {
        None
    }

    fn response_node_id(&self, _response: &super::QueryResponse) -> Option<ResponseNodeId> {
        Some(*self)
    }
}

impl ResponseIdLookup for NodeID<'_> {
    fn node_id(&self) -> Option<ArcIntern<String>> {
        Some(ArcIntern::new(self.as_ref().to_string()))
    }

    fn response_node_id(&self, response: &super::QueryResponse) -> Option<ResponseNodeId> {
        response.entity_ids.get(&self.node_id().unwrap()).copied()
    }
}

impl ResponseIdLookup for ArcIntern<String> {
    fn node_id(&self) -> Option<ArcIntern<String>> {
        Some(self.clone())
    }

    fn response_node_id(&self, response: &super::QueryResponse) -> Option<ResponseNodeId> {
        response.entity_ids.get(self).copied()
    }
}

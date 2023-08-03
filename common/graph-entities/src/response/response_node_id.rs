use internment::ArcIntern;
use serde::{Deserialize, Serialize};

use super::EntityId;
use crate::NodeID;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, Serialize, Deserialize)]
/// The identifier of a node within a [`QueryResponse`].
///
/// This ID is used to lookup other nodes in the graph and to link
/// a node to its children
pub struct ResponseNodeId(pub(crate) u32);

pub trait ToEntityId {
    fn entity_id(&self) -> EntityId;
}

impl ToEntityId for NodeID<'_> {
    fn entity_id(&self) -> EntityId {
        self.clone().into()
    }
}

impl ToEntityId for EntityId {
    fn entity_id(&self) -> EntityId {
        self.clone()
    }
}

impl ToEntityId for ArcIntern<String> {
    fn entity_id(&self) -> EntityId {
        self.clone().into()
    }
}

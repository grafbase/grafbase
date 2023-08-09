use internment::ArcIntern;
use serde::{Deserialize, Serialize};

use crate::NodeID;

/// The ID of an entity in the database.
///
/// This is basically just a stringified `crate::id::NodeID` but wrapped in a type
/// for clarity
///
/// Named without the word Node because we're already working with ResponseNodes which
/// have their own IDs _and_ sometimes contain one of these EntityIds as well.
///
/// We should maybe think about our terminology sometime to make this less confusing
#[derive(PartialEq, Eq, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct EntityId(ArcIntern<String>);

impl EntityId {
    pub fn into_inner(self) -> ArcIntern<String> {
        self.0
    }
}

impl From<NodeID<'_>> for EntityId {
    fn from(value: NodeID<'_>) -> Self {
        EntityId(ArcIntern::new(value.to_string()))
    }
}

impl From<&NodeID<'_>> for EntityId {
    fn from(value: &NodeID<'_>) -> Self {
        EntityId(ArcIntern::new(value.to_string()))
    }
}

impl From<ArcIntern<String>> for EntityId {
    fn from(value: ArcIntern<String>) -> Self {
        EntityId(value)
    }
}

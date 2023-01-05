use crate::NodeID;
use getrandom::getrandom;
use internment::ArcIntern;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Hash, Serialize, Deserialize)]
/// A node identifier within a particular [`Tree`].
///
/// This ID is used to get [`Node`] references from an [`Tree`].
/// Cheap to clone.
pub enum ResponseNodeId {
    // TODO: For compaction, it could be interesting to have a counter of reference for this ID,
    // (so let's switch the id: u64 by an ArcIntern)
    /// An ID which describe an Internal Node which isn't an Entity, like a primitive.
    Internal {
        id: u64,
    },
    // An ID which describe an Entity
    NodeID(ArcIntern<String>),
}

impl ResponseNodeId {
    /// Generate a new Internal NodeID
    pub fn internal() -> Self {
        let mut buf = [0u8; 8];
        getrandom(&mut buf[..]).expect("Shouldn't fail");

        Self::Internal {
            id: u64::from_be_bytes(buf),
        }
    }

    pub fn node<'a, S: AsRef<NodeID<'a>>>(id: S) -> Self {
        Self::NodeID(ArcIntern::new(id.as_ref().to_string()))
    }

    pub fn get_node_id(&self) -> Option<ArcIntern<String>> {
        if let Self::NodeID(arc) = self {
            Some(arc.clone())
        } else {
            None
        }
    }
}

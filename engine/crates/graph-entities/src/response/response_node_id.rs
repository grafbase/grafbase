use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, Serialize, Deserialize)]
/// The identifier of a node within a [`QueryResponse`].
///
/// This ID is used to lookup other nodes in the graph and to link
/// a node to its children
pub struct ResponseNodeId(pub(crate) u32);

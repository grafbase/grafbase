mod loader;

use ulid::Ulid;

pub use loader::*;

#[derive(Debug, Clone)]
pub enum Graph {
    FromGraphRef {
        branch_id: Ulid,
        version_id: Ulid,
        sdl: String,
    },
    FromText {
        sdl: String,
    },
}

impl Graph {
    pub fn sdl(&self) -> &str {
        match self {
            Graph::FromGraphRef { sdl, .. } => sdl,
            Graph::FromText { sdl, .. } => sdl,
        }
    }

    pub fn branch_id(&self) -> Option<Ulid> {
        match self {
            Graph::FromGraphRef { branch_id, .. } => Some(*branch_id),
            Graph::FromText { .. } => None,
        }
    }

    pub fn version_id(&self) -> Option<Ulid> {
        match self {
            Graph::FromGraphRef { version_id, .. } => Some(*version_id),
            Graph::FromText { .. } => None,
        }
    }
}

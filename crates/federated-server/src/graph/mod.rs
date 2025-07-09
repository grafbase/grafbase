mod loader;

use std::path::{Path, PathBuf};

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
        parent_dir: Option<PathBuf>,
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

    pub fn parent_dir_path(&self) -> Option<&Path> {
        match self {
            Graph::FromGraphRef { .. } => None,
            Graph::FromText { parent_dir, .. } => parent_dir.as_deref(),
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

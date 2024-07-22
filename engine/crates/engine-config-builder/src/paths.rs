use std::path::{Path, PathBuf};

use crate::config::PathId;

/// Very simple implementation of string interning.
#[derive(Default)]
pub(crate) struct Paths<'a>(indexmap::IndexSet<&'a Path>);

impl<'a> Paths<'a> {
    /// Interns a str, avoiding allocation if the same string has already been
    /// interned.
    pub(crate) fn intern(&mut self, path: &'a Path) -> PathId {
        if let Some(idx) = self.0.get_index_of(path) {
            return PathId(idx);
        }

        PathId(self.0.insert_full(path).0)
    }

    pub fn into_vec(self) -> Vec<PathBuf> {
        self.0.into_iter().map(|path| path.to_path_buf()).collect()
    }
}

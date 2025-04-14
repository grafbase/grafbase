use std::path::PathBuf;

pub use extension::*;

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct ExtensionId(u16);

#[derive(Debug, Default, id_derives::IndexedFields)]
pub struct ExtensionCatalog {
    #[indexed_by(ExtensionId)]
    extensions: Vec<Extension>,
}

#[derive(Clone, Debug)]
pub struct Extension {
    pub manifest: Manifest,
    pub wasm_path: PathBuf,
}

impl ExtensionCatalog {
    /// Function must be deterministic and always return the same result for a given extension::Id.
    pub fn find_compatible_extension(&self, id: &extension::Id) -> Option<ExtensionId> {
        self.extensions
            .iter()
            .enumerate()
            .find(|(_, existing)| existing.manifest.id.is_compatible_with(id))
            .map(|(ix, _)| ix.into())
    }

    pub fn get_id_by_name(&self, name: &str) -> Option<ExtensionId> {
        self.extensions
            .iter()
            .enumerate()
            .find(|(_, existing)| existing.manifest.name() == name)
            .map(|(ix, _)| ix.into())
    }

    pub fn push(&mut self, extension: Extension) -> ExtensionId {
        self.extensions.push(extension);
        (self.extensions.len() - 1).into()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Extension> {
        self.extensions.iter()
    }

    pub fn iter_with_id(&self) -> impl ExactSizeIterator<Item = (ExtensionId, &Extension)> {
        self.extensions.iter().enumerate().map(|(ix, ext)| (ix.into(), ext))
    }

    pub fn len(&self) -> usize {
        self.extensions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

use std::path::PathBuf;

pub use extension::*;

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct ExtensionId(u16);

#[derive(Debug, Clone, Default, id_derives::IndexedFields)]
pub struct ExtensionCatalog {
    #[indexed_by(ExtensionId)]
    extensions: Vec<Extension>,
}

#[derive(Clone, Debug)]
pub struct Extension {
    pub config_key: String,
    pub manifest: Manifest,
    pub wasm_path: PathBuf,
}

impl ExtensionCatalog {
    /// Function must be deterministic and always return the same result for a given extension::Id.
    pub fn find_compatible_extension(
        &self,
        link_url: &str,
        name: Option<&str>,
        version: Option<&semver::VersionReq>,
    ) -> Option<ExtensionId> {
        // First look for explicitly associated link URLs.
        if let Some((ix, _)) = self.extensions.iter().enumerate().find(|(_, ext)| {
            ext.manifest.associated_link_urls.iter().any(|prefix| {
                link_url.starts_with(prefix) && {
                    match (name, version) {
                        (Some(name), Some(version)) => ext.manifest.id.is_compatible_with(name, version),
                        (Some(name), None) => ext.manifest.name() == name,
                        _ => true,
                    }
                }
            })
        }) {
            return Some(ix.into());
        }
        match (name, version) {
            (Some(name), Some(version)) => self
                .extensions
                .iter()
                .enumerate()
                .find(|(_, existing)| existing.manifest.id.is_compatible_with(name, version))
                .map(|(ix, _)| ix.into()),
            (Some(name), None) => self.get_id_by_name(name),
            _ => None,
        }
    }

    pub fn get_id_by_name(&self, name: &str) -> Option<ExtensionId> {
        self.extensions
            .iter()
            .enumerate()
            .find(|(_, existing)| existing.manifest.name() == name)
            .map(|(ix, _)| ix.into())
    }

    pub fn get_id_by_config_key(&self, config_key: &str) -> Option<ExtensionId> {
        self.extensions
            .iter()
            .enumerate()
            .find(|(_, existing)| existing.config_key == config_key)
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

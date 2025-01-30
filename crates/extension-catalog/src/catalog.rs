use std::path::PathBuf;

pub use extension::*;

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct ExtensionId(u16);

#[derive(Default, id_derives::IndexedFields)]
pub struct ExtensionCatalog {
    #[indexed_by(ExtensionId)]
    extensions: Vec<Extension>,
}

pub struct Extension {
    pub id: Id,
    pub manifest: Manifest,
    pub wasm_path: PathBuf,
}

#[derive(Default)]
pub enum ExtensionDirectiveKind {
    #[default]
    Unknown,
    FieldResolver,
}

impl ExtensionDirectiveKind {
    pub fn is_field_resolver(&self) -> bool {
        matches!(self, Self::FieldResolver)
    }
}

impl ExtensionCatalog {
    /// Function must be deterministic and always return the same result for a given extension::Id.
    pub fn find_compatible_extension(&self, id: &extension::Id) -> Option<ExtensionId> {
        self.extensions
            .iter()
            .enumerate()
            .find(|(_, existing)| existing.id.is_compatible_with(id))
            .map(|(ix, _)| ix.into())
    }

    pub fn get_directive_kind(&self, id: ExtensionId, name: &str) -> ExtensionDirectiveKind {
        match &self[id].manifest.kind {
            extension::Kind::FieldResolver(FieldResolver { resolver_directives })
                if resolver_directives.iter().any(|dir| dir == name) =>
            {
                ExtensionDirectiveKind::FieldResolver
            }
            _ => ExtensionDirectiveKind::Unknown,
        }
    }

    pub fn push(&mut self, extension: Extension) -> ExtensionId {
        self.extensions.push(extension);
        (self.extensions.len() - 1).into()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Extension> {
        self.extensions.iter()
    }

    pub fn len(&self) -> usize {
        self.extensions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

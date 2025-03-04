use std::path::PathBuf;

pub use extension::*;

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct ExtensionId(u16);

#[derive(Debug, Default, id_derives::IndexedFields)]
pub struct ExtensionCatalog {
    #[indexed_by(ExtensionId)]
    extensions: Vec<Extension>,
}

#[derive(Debug)]
pub struct Extension {
    pub manifest: Manifest,
    pub wasm_path: PathBuf,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ExtensionDirectiveKind {
    #[default]
    Unknown,
    FieldResolver,
    Authorization,
}

impl ExtensionDirectiveKind {
    pub fn is_resolver(&self) -> bool {
        matches!(self, Self::FieldResolver)
    }

    pub fn is_authorization(&self) -> bool {
        matches!(self, Self::Authorization)
    }
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

    pub fn get_directive_kind(&self, id: ExtensionId, name: &str) -> ExtensionDirectiveKind {
        match &self[id].manifest.kind {
            extension::Kind::Resolver(ResolverKind { resolver_directives }) => {
                if let Some(directives) = resolver_directives {
                    directives
                        .iter()
                        .any(|dir| dir == name)
                        .then_some(ExtensionDirectiveKind::FieldResolver)
                        .unwrap_or_default()
                } else {
                    ExtensionDirectiveKind::FieldResolver
                }
            }
            extension::Kind::Authorization(AuthorizationKind {
                authorization_directives: directives,
            }) => {
                if let Some(directives) = directives {
                    directives
                        .iter()
                        .any(|dir| dir == name)
                        .then_some(ExtensionDirectiveKind::Authorization)
                        .unwrap_or_default()
                } else {
                    ExtensionDirectiveKind::Authorization
                }
            }
            extension::Kind::Authentication(_) => Default::default(),
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

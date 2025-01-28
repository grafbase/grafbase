use super::ExtensionId;

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

/// Used during the engine::Schema building to identify and treat correctly extension directives.
pub trait ExtensionCatalog: Send + Sync {
    fn find_compatible_extension(&self, id: &extension::Id) -> Option<ExtensionId>;
    fn get_directive_kind(&self, id: ExtensionId, name: &str) -> ExtensionDirectiveKind;
}

impl ExtensionCatalog for () {
    fn find_compatible_extension(&self, _id: &extension::Id) -> Option<ExtensionId> {
        None
    }
    fn get_directive_kind(&self, _id: ExtensionId, _name: &str) -> ExtensionDirectiveKind {
        Default::default()
    }
}

/// Avoids dealing with lifetimes while building the Schema
impl<T: ExtensionCatalog> ExtensionCatalog for &T {
    fn find_compatible_extension(&self, id: &extension::Id) -> Option<ExtensionId> {
        (*self).find_compatible_extension(id)
    }

    fn get_directive_kind(&self, id: ExtensionId, name: &str) -> ExtensionDirectiveKind {
        (*self).get_directive_kind(id, name)
    }
}

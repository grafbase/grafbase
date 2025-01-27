use super::ExtensionId;

pub trait ExtensionCatalog: Send + Sync {
    fn find_compatible_extension(&self, id: &extension::Id) -> Option<ExtensionId>;
}

impl ExtensionCatalog for () {
    fn find_compatible_extension(&self, _id: &extension::Id) -> Option<ExtensionId> {
        None
    }
}

impl<T: ExtensionCatalog> ExtensionCatalog for &T {
    fn find_compatible_extension(&self, id: &extension::Id) -> Option<ExtensionId> {
        (*self).find_compatible_extension(id)
    }
}

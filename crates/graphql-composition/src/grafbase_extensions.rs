/// A Grafbase extension registered for composition.
pub struct LoadedExtension {
    pub(crate) url: String,
    /// The unique name of the extension.
    pub(crate) name: String,
}

impl LoadedExtension {
    /// Construct a [LoadedExtension].
    pub fn new(url: String, name: String) -> Self {
        Self { url, name }
    }
}

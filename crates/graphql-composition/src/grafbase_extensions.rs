/// A Grafbase extension registered for composition.
#[derive(Debug)]
#[doc(hidden)]
pub struct LoadedExtension {
    /// The URL of the extension, which can be a remote URL or a local file path.
    pub url: String,
    /// The unique name of the extension.
    pub name: String,
}

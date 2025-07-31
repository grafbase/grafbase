/// A Grafbase extension registered for composition.
#[derive(Debug)]
pub struct LoadedExtension {
    /// The URL of the extension, which can be a remote URL or a local file path.
    pub link_url: String,
    /// URL to use in the federated SDL
    pub url: url::Url,
    /// The unique name of the extension.
    pub name: String,
    /// The version of the extension.
    pub version: String,
}

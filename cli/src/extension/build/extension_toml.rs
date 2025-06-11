use extension::EventFilter;
use semver::Version;
use serde_valid::Validate;

#[derive(serde::Deserialize)]
pub struct ExtensionToml {
    pub extension: ExtensionTomlExtension,
    #[serde(default)]
    pub directives: ExtensionTomlDirectives,
    #[serde(default)]
    pub permissions: ExtensionTomlPermissions,
    #[serde(default)]
    pub hooks: ExtensionTomlHooks,
}

#[derive(Default, serde::Deserialize)]
pub struct ExtensionTomlHooks {
    pub events: Option<EventFilter>,
}

#[derive(Default, serde::Deserialize)]
pub struct ExtensionTomlDirectives {
    pub definitions: Option<String>,
    pub field_resolvers: Option<Vec<String>>,
    pub resolvers: Option<Vec<String>>,
    pub authorization: Option<Vec<String>>,
}

#[derive(Default, serde::Deserialize)]
pub struct ExtensionTomlPermissions {
    #[serde(default)]
    pub network: bool,
    #[serde(default)]
    pub stdout: bool,
    #[serde(default)]
    pub stderr: bool,
    #[serde(default)]
    pub environment_variables: bool,
}

#[derive(serde::Deserialize, Validate)]
pub struct ExtensionTomlExtension {
    #[validate(pattern = "^[a-z0-9-]+$")]
    pub name: String,
    pub version: Version,
    // backwards compatibility for now.
    #[serde(alias = "kind")]
    pub r#type: ExtensionType,
    pub description: String,
    pub homepage_url: Option<url::Url>,
    pub repository_url: Option<url::Url>,
    pub license: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionType {
    Resolver,
    Authentication,
    Authorization,
    SelectionSetResolver,
    Hooks,
}

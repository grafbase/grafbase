use super::schema;

#[derive(Debug, cynic::QueryFragment)]
pub struct Extension {
    pub name: String,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct ExtensionVersion {
    pub extension: Extension,
    pub version: semver::Version,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct ExtensionDoesNotExistError {
    __typename: String,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct ExtensionVersionDoesNotExistError {
    __typename: String,
}

#[derive(Debug, cynic::InlineFragments)]
pub enum ExtensionVersionMatch {
    ExtensionVersion(ExtensionVersion),
    ExtensionDoesNotExistError(#[expect(unused)] ExtensionDoesNotExistError),
    ExtensionVersionDoesNotExistError(#[expect(unused)] ExtensionVersionDoesNotExistError),
    #[cynic(fallback)]
    Unknown(String),
}

#[derive(Debug, cynic::InputObject)]
pub(crate) struct ExtensionVersionRequirement {
    pub extension_name: String,
    pub version: semver::VersionReq,
}

#[derive(Debug, cynic::QueryVariables)]
pub struct ExtensionVersionsByVersionRequirementVariables {
    pub(crate) requirements: Vec<ExtensionVersionRequirement>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ExtensionVersionsByVersionRequirementVariables")]
pub(crate) struct ExtensionVersionsByVersionRequirement {
    #[arguments(requirements: $requirements)]
    pub(crate) extension_versions_by_version_requirement: Option<Vec<ExtensionVersionMatch>>,
}

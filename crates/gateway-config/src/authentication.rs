/// Configures the GraphQL server JWT authentication
#[derive(Default, Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct AuthenticationConfig {
    pub default: Option<DefaultAuthenticationBehavior>,
    pub protected_resources: AuthenticationResources,
}

#[derive(Default, Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct AuthenticationResources {
    pub graphql: AuthenticationResourcesConfig,
    pub mcp: AuthenticationResourcesConfig,
}

#[derive(Default, Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct AuthenticationResourcesConfig {
    pub extensions: Option<Vec<String>>,
    pub default: Option<DefaultAuthenticationBehavior>,
}

#[derive(Debug, PartialEq, serde::Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum DefaultAuthenticationBehavior {
    Anonymous,
    Deny,
}

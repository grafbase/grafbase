#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionPermission {
    Network,
    Stdout,
    Stderr,
    EnvironmentVariables,
}

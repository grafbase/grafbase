#[derive(Debug, Default, serde::Deserialize, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct PrewarmingConfig {
    pub enabled: bool,
}

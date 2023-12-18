use crate::node::NodeError;

#[derive(thiserror::Error, Debug, Clone)]
pub enum ConfigError {
    #[error("{0}")]
    Io(String),

    #[error("non utf-8 path used for project")]
    ProjectPath,

    /// returned if the schema parser errors
    #[error("{0}")]
    ParseSchema(String),

    #[error(transparent)]
    NodeError(#[from] NodeError),

    /// returned if the typescript config parser command exits unsuccessfully
    #[error("could not load grafbase/grafbase.config.ts\nCaused by: {0}")]
    LoadTsConfig(String),
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        ConfigError::Io(value.to_string())
    }
}

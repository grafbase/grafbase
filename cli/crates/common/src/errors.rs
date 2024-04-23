use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommonError {
    /// returned if the current directory path cannot be read
    #[error("could not read the current path")]
    ReadCurrentDirectory,
    /// returned if the grafbase directory cannot be found
    #[error(
        "could not find grafbase/grafbase.config.ts or grafbase/schema.graphql in the current or any parent directory"
    )]
    FindGrafbaseDirectory,
    /// returned if the home directory for the current user could not be found
    #[error("could not find the home directory for the current user")]
    FindHomeDirectory,
    /// returned if analytics.json could not be written
    #[error("could not write the analytics data file\nCaused by: {0}")]
    WriteAnalyticsDataFile(std::io::Error),
    /// returned if analytics.json could not be read
    #[error("could not read the analytics data file\nCaused by: {0}")]
    ReadAnalyticsDataFile(std::io::Error),
    /// returned if analytics.json is corrupt
    #[error("the analytics data file is corrupt")]
    CorruptAnalyticsDataFile,
    /// returned if ~/.grafbase could not be created
    #[error("could not create '~/.grafbase'\nCaused by: {0}")]
    CreateUserDotGrafbaseFolder(std::io::Error),
    #[error("could not open the project's 'package.json':\nCaused by: {0}")]
    AccessPackageJson(std::io::Error),
    #[error("could not serialize the project's 'package.json':\nCaused by: {0}")]
    SerializePackageJson(serde_json::Error),
    #[error("could not execute 'npm init':\nCaused by: {0}")]
    NpmInitError(std::io::Error),
    #[error("could not open file '{0}':\nCaused by: {1}")]
    RegistryRead(std::path::PathBuf, std::io::Error),
    #[error("could not deserialize to json the contents of '{0}':\nCaused by: {1}")]
    RegistryDeserialization(std::path::PathBuf, serde_json::Error),
    #[error(transparent)]
    BunNotFound(#[from] BunNotFound),
}

#[derive(Debug, thiserror::Error, Clone, Copy)]
#[error("Could not find a `bun` executable in PATH. Bun is required in order to evaluate your TypeScript configuration. Please install `bun` or run `nix shell nixpkgs#bun`.")]
pub struct BunNotFound;

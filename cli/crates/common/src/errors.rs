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
    #[error("could not write the analytics data file\ncaused by: {0}")]
    WriteAnalyticsDataFile(std::io::Error),
    /// returned if analytics.json could not be read
    #[error("could not read the analytics data file\ncaused by: {0}")]
    ReadAnalyticsDataFile(std::io::Error),
    /// returned if analytics.json is corrupt
    #[error("the analytics data file is corrupt")]
    CorruptAnalyticsDataFile,
    /// returned if ~/.grafbase could not be created
    #[error("could not create '~/.grafbase'\ncaused by: {0}")]
    CreateUserDotGrafbaseFolder(std::io::Error),
}

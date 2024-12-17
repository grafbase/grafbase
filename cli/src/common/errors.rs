use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum CommonError {
    /// returned if the home directory for the current user could not be found
    #[error("could not find the home directory for the current user")]
    FindHomeDirectory,
    #[error("encountered an invalid dashboard URL")]
    InvalidDashboardUrl,
    /// returned if ~/.grafbase could not be read
    #[error("could not read '~/.grafbase'\nCaused by: {0}")]
    ReadUserDotGrafbaseFolder(io::Error),
    /// returned if ~/.grafbase/credentials.json could not be read
    #[error("could not read '~/.grafbase/credentials.json'\nCaused by: {0}")]
    ReadCredentialsFile(io::Error),
    /// returned if the contents of the credential file are corrupt
    #[error("could not complete the action as your credential file is corrupt")]
    CorruptCredentialsFile,
}

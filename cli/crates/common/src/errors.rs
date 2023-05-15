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
}

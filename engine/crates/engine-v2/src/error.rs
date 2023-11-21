use crate::{
    plan::PrepareError,
    request::{BindError, ParseError},
    response::GraphqlError,
};

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Bind(#[from] BindError),
    #[error(transparent)]
    Prepare(#[from] PrepareError),
}

impl From<EngineError> for GraphqlError {
    fn from(err: EngineError) -> Self {
        match err {
            EngineError::Bind(err) => err.into(),
            EngineError::Prepare(err) => err.into(),
            EngineError::Parse(err) => err.into(),
        }
    }
}

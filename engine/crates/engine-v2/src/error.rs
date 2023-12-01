use crate::{
    request::{BindError, ParseError},
    response::ServerError,
};

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Bind(#[from] BindError),
}

impl From<EngineError> for ServerError {
    fn from(err: EngineError) -> Self {
        match err {
            EngineError::Bind(err) => err.into(),
            EngineError::Parse(err) => err.into(),
        }
    }
}

use error::GraphqlError;

#[derive(Debug, thiserror::Error)]
pub(super) enum DeserError {
    #[error(transparent)]
    Json(#[from] sonic_rs::Error),
    #[error(transparent)]
    Cbor(#[from] minicbor_serde::error::DecodeError),
    #[error("{}", .0.message)]
    Graphql(GraphqlError),
}

impl From<GraphqlError> for DeserError {
    fn from(err: GraphqlError) -> Self {
        DeserError::Graphql(err)
    }
}

impl From<DeserError> for GraphqlError {
    fn from(err: DeserError) -> Self {
        match err {
            DeserError::Graphql(err) => err,
            DeserError::Json(_) | DeserError::Cbor(_) => GraphqlError::invalid_subgraph_response(),
        }
    }
}
